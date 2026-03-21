use anyhow::Result;
use std::path::Path;
use crate::storage::{Storage, RepoInfo};
use crate::parser::Parser;
use crate::route_extractor::extract_routes;
use crate::embedding::EmbeddingEngine;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use sha2::{Sha256, Digest};
use std::fs;
use rusqlite::params;

pub struct Indexer {
    storage: Storage,
    parser: Parser,
    embedding_engine: Option<EmbeddingEngine>,
}

impl Indexer {
    pub fn new(storage: Storage) -> Result<Self> {
        let parser = Parser::new()?;
        Ok(Self {
            storage,
            parser,
            embedding_engine: None, 
        })
    }

    pub fn index_folder(&mut self, path: &Path, name: Option<String>, ignore_patterns: Vec<String>, json: bool) -> Result<RepoInfo> {
        let path = fs::canonicalize(path)?;
        let repo_name = name.unwrap_or_else(|| {
            path.file_name().unwrap().to_string_lossy().to_string()
        });

        let db_path = self.storage.get_db_path(&repo_name);
        let conn = self.storage.open_repo_db(&db_path)?;

        let mut repo_info = RepoInfo {
            name: repo_name.clone(),
            root_path: path.clone(),
            symbol_count: None,
            db_path: db_path.clone(),
            last_indexed: None,
        };

        self.storage.register_repo(&repo_info)?;

        // Build the walker with ignore support
        let mut builder = WalkBuilder::new(&path);
        builder
            .hidden(false)           // Include hidden files (user can ignore explicitly)
            .git_ignore(true)        // Respect .gitignore
            .git_global(true)        // Respect global gitignore
            .git_exclude(true)       // Respect .git/info/exclude
            .ignore(true)            // Respect .ignore files
            .add_custom_ignore_filename(".rustdexignore");  // Our custom ignore file

        // Add CLI-provided ignore patterns using overrides
        // This uses glob patterns where patterns starting with '!' are exclusions
        if !ignore_patterns.is_empty() {
            let mut override_builder = OverrideBuilder::new(&path);
            for pattern in &ignore_patterns {
                // Convert gitignore-style pattern to override pattern
                // Override patterns use '!' prefix for exclusions (inverse of gitignore)
                // So a gitignore pattern like "*.log" becomes "!*.log" in override terms
                let override_pattern = format!("!{}", pattern);
                override_builder.add(&override_pattern)?;
            }
            builder.overrides(override_builder.build()?);
        }

        // Skip binary file extensions
        let skip_exts = vec![".pyc", ".so", ".dylib", ".dll", ".exe", ".bin", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".db", ".sqlite", ".pdf", ".zip", ".tar", ".gz", ".mp3", ".mp4", ".mov", ".avi", ".wav", ".ico", ".ttf", ".woff", ".woff2", ".eot", ".otf"];

        let mut symbol_count = 0;

        for result in builder.build() {
            let entry = match result {
                Ok(e) => e,
                Err(err) => {
                    if !json {
                        eprintln!("Warning: {}", err);
                    }
                    continue;
                }
            };

            if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                continue;
            }

            let file_path = entry.path();
            let ext = file_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
            if skip_exts.contains(&format!(".{}", ext).as_str()) {
                continue;
            }

            let rel_path = file_path.strip_prefix(&path)?.to_string_lossy().to_string();
            
            // Read file content
            let bytes = match fs::read(file_path) {
                Ok(b) => b,
                Err(err) => {
                    if !json {
                        eprintln!("Warning: Could not read {}: {}", rel_path, err);
                    }
                    continue;
                }
            };
            let source = String::from_utf8_lossy(&bytes).to_string();
            let file_hash = calculate_hash(&source);

            // Check if file has changed
            let mut stmt = conn.prepare("SELECT hash FROM files WHERE repo = ? AND path = ?")?;
            let mut rows = stmt.query(params![repo_name, rel_path])?;
            if let Some(row) = rows.next()? {
                let old_hash: String = row.get(0)?;
                if old_hash == file_hash {
                    continue;
                }
            }

            if !json {
                println!("Indexing {}...", rel_path);
            }

            // Remove old data for this file
            conn.execute("DELETE FROM symbols WHERE repo = ? AND file = ?", params![repo_name, rel_path])?;
            conn.execute("DELETE FROM routes WHERE repo = ? AND file = ?", params![repo_name, rel_path])?;

            // Parse and store symbols
            let symbols = self.parser.parse_file(&source, &ext, &repo_name, &rel_path)?;
            for mut symbol in symbols {
                conn.execute(
                    "INSERT INTO symbols (repo, file, name, kind, start_byte, end_byte, signature, docstring)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        symbol.repo,
                        symbol.file,
                        symbol.name,
                        symbol.kind,
                        symbol.start_byte,
                        symbol.end_byte,
                        symbol.signature,
                        symbol.docstring,
                    ],
                )?;
                let symbol_id = conn.last_insert_rowid();
                symbol.id = Some(symbol_id);
                symbol_count += 1;

                // Generate embedding
                if self.embedding_engine.is_none() {
                    if !json {
                        println!("Loading embedding engine...");
                    }
                    self.embedding_engine = Some(EmbeddingEngine::new()?);
                }
                if let Some(engine) = &self.embedding_engine {
                    let text = format!("{}\n{}\n{}", symbol.signature.unwrap_or_default(), symbol.docstring.unwrap_or_default(), symbol.name);
                    if let Ok(vec) = engine.embed(&text) {
                        let bytes: Vec<u8> = vec.iter().flat_map(|f| f.to_le_bytes().to_vec()).collect();
                        conn.execute("UPDATE symbols SET embedding = ? WHERE id = ?", params![bytes, symbol_id])?;
                    }
                }
            }

            // Extract and store routes
            let routes = extract_routes(source.as_bytes(), &ext, &repo_name, &rel_path);
            for route in routes {
                conn.execute(
                    "INSERT INTO routes (repo, file, method, path, handler, start_byte, end_byte)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                    params![
                        route.repo,
                        route.file,
                        route.method,
                        route.path,
                        route.handler,
                        route.start_byte,
                        route.end_byte,
                    ],
                )?;
            }

            // Record file as indexed
            conn.execute(
                "INSERT OR REPLACE INTO files (repo, path, hash, indexed_at) VALUES (?, ?, ?, datetime('now'))",
                params![repo_name, rel_path, file_hash],
            )?;
        }

        if !json {
            println!("Indexed {} symbols.", symbol_count);
        }
        repo_info.symbol_count = Some(symbol_count);
        repo_info.last_indexed = Some(chrono::Utc::now().naive_utc());

        Ok(repo_info)
    }
}

fn calculate_hash(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    hex::encode(hasher.finalize())
}