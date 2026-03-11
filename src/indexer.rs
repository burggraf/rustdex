use anyhow::Result;
use std::path::Path;
use crate::storage::{Storage, RepoInfo};
use crate::parser::Parser;
use crate::route_extractor::extract_routes;
use crate::embedding::EmbeddingEngine;
use walkdir::WalkDir;
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
            embedding_engine: None, // Lazy load
        })
    }

    pub fn index_folder(&mut self, path: &Path, name: Option<String>, json: bool) -> Result<RepoInfo> {
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

        let skip_dirs = vec![".git", "node_modules", "__pycache__", "venv", ".venv", "target"];
        let skip_exts = vec![".pyc", ".so", ".dylib", ".dll", ".exe", ".bin", ".png", ".jpg", ".db", ".sqlite"];

        let mut symbol_count = 0;

        for entry in WalkDir::new(&path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !skip_dirs.contains(&name.as_ref())
            })
        {
            let entry = entry?;
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let ext = file_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
                if skip_exts.contains(&format!(".{}", ext).as_str()) {
                    continue;
                }

                let rel_path = file_path.strip_prefix(&path)?.to_string_lossy().to_string();
                let source = fs::read_to_string(file_path)?;
                let file_hash = calculate_hash(&source);

                // Check if already indexed and unchanged
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

                // Delete old records for this file
                conn.execute("DELETE FROM symbols WHERE repo = ? AND file = ?", params![repo_name, rel_path])?;
                conn.execute("DELETE FROM routes WHERE repo = ? AND file = ?", params![repo_name, rel_path])?;

                // Extract symbols
                let symbols = self.parser.parse_file(&source, &ext, &repo_name, &rel_path)?;
                for mut symbol in symbols {
                    // Upsert symbol
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

                // Extract routes
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

                // Upsert file hash
                conn.execute(
                    "INSERT OR REPLACE INTO files (repo, path, hash, indexed_at) VALUES (?, ?, ?, datetime('now'))",
                    params![repo_name, rel_path, file_hash],
                )?;
            }
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
