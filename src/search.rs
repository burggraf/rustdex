use anyhow::{Result, Context};
use crate::storage::{Storage, Symbol, RouteInfo, RepoInfo};
use crate::embedding::EmbeddingEngine;
use std::fs;

pub struct Searcher {
    storage: Storage,
    embedding_engine: Option<EmbeddingEngine>,
}

impl Searcher {
    pub fn new(storage: Storage) -> Result<Self> {
        Ok(Self {
            storage,
            embedding_engine: None,
        })
    }

    pub fn search_symbols(
        &self,
        query: &str,
        repo: Option<&str>,
        kind: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Symbol>> {
        let repos = self.get_target_repos(repo)?;
        let mut all_results = Vec::new();

        for repo_info in repos {
            let conn = self.storage.open_repo_db(&repo_info.db_path)?;
            let mut sql = "SELECT id, repo, file, name, kind, start_byte, end_byte, signature, docstring FROM symbols WHERE name LIKE ?".to_string();
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(format!("%{}%", query))];

            if let Some(k) = kind {
                sql.push_str(" AND kind = ?");
                params_vec.push(Box::new(k.to_string()));
            }

            sql.push_str(" LIMIT ?");
            params_vec.push(Box::new(limit as i64));

            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
                Ok(Symbol {
                    id: Some(row.get(0)?),
                    repo: row.get(1)?,
                    file: row.get(2)?,
                    name: row.get(3)?,
                    kind: row.get(4)?,
                    start_byte: row.get(5)?,
                    end_byte: row.get(6)?,
                    signature: row.get(7)?,
                    docstring: row.get(8)?,
                    embedding: None,
                })
            })?;

            for row in rows {
                all_results.push(row?);
            }
        }

        all_results.sort_by(|a, b| a.name.len().cmp(&b.name.len())); // Shortest name matches first
        Ok(all_results.into_iter().take(limit).collect())
    }

    pub fn search_semantic(
        &mut self,
        query: &str,
        repo: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(Symbol, f32)>> {
        if self.embedding_engine.is_none() {
            self.embedding_engine = Some(EmbeddingEngine::new()?);
        }
        let query_vec = self.embedding_engine.as_ref().unwrap().embed(query)?;

        let repos = self.get_target_repos(repo)?;
        let mut scored_results = Vec::new();

        for repo_info in repos {
            let conn = self.storage.open_repo_db(&repo_info.db_path)?;
            let mut stmt = conn.prepare("SELECT id, repo, file, name, kind, start_byte, end_byte, signature, docstring, embedding FROM symbols WHERE embedding IS NOT NULL")?;
            
            let rows = stmt.query_map([], |row| {
                let id: i64 = row.get(0)?;
                let repo_name: String = row.get(1)?;
                let file: String = row.get(2)?;
                let name: String = row.get(3)?;
                let kind: String = row.get(4)?;
                let start_byte: usize = row.get(5)?;
                let end_byte: usize = row.get(6)?;
                let signature: Option<String> = row.get(7)?;
                let docstring: Option<String> = row.get(8)?;
                let embedding_bytes: Vec<u8> = row.get(9)?;

                let symbol = Symbol {
                    id: Some(id),
                    repo: repo_name,
                    file,
                    name,
                    kind,
                    start_byte,
                    end_byte,
                    signature,
                    docstring,
                    embedding: Some(embedding_bytes.clone()),
                };

                // Convert bytes back to f32 vec
                let stored_vec: Vec<f32> = embedding_bytes
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
                    .collect();

                let score = dot_product(&query_vec, &stored_vec);
                Ok((symbol, score))
            })?;

            for row in rows {
                scored_results.push(row?);
            }
        }

        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored_results.into_iter().take(limit).collect())
    }

    pub fn search_routes(
        &self,
        repo: &str,
        method: Option<&str>,
        path: Option<&str>,
        limit: usize,
    ) -> Result<Vec<RouteInfo>> {
        let repo_info = self.storage.get_repo_info(repo)?.context("Repo not found")?;
        let conn = self.storage.open_repo_db(&repo_info.db_path)?;

        let mut sql = "SELECT id, repo, file, method, path, handler, start_byte, end_byte FROM routes WHERE repo = ?".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(repo.to_string())];

        if let Some(m) = method {
            sql.push_str(" AND method = ?");
            params_vec.push(Box::new(m.to_uppercase()));
        }

        if let Some(p) = path {
            sql.push_str(" AND path LIKE ?");
            params_vec.push(Box::new(format!("%{}%", p)));
        }

        sql.push_str(" LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(RouteInfo {
                id: Some(row.get(0)?),
                repo: row.get(1)?,
                file: row.get(2)?,
                method: row.get(3)?,
                path: row.get(4)?,
                handler: row.get(5)?,
                start_byte: row.get(6)?,
                end_byte: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    #[allow(dead_code)]
    pub fn get_symbol_source(&self, repo: &str, file: &str, start_byte: usize, end_byte: usize) -> Result<String> {
        let repo_info = self.storage.get_repo_info(repo)?.context("Repo not found")?;
        let full_path = repo_info.root_path.join(file);
        let content = fs::read(&full_path)?;
        
        if start_byte < content.len() && end_byte <= content.len() {
            Ok(String::from_utf8_lossy(&content[start_byte..end_byte]).to_string())
        } else {
            anyhow::bail!("Byte offsets out of bounds")
        }
    }

    fn get_target_repos(&self, repo_name: Option<&str>) -> Result<Vec<RepoInfo>> {
        if let Some(name) = repo_name {
            let info = self.storage.get_repo_info(name)?.context("Repo not found")?;
            Ok(vec![info])
        } else {
            self.storage.list_repos()
        }
    }
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}
