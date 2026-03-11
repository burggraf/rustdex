use anyhow::{Result, Context};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Symbol {
    pub id: Option<i64>,
    pub repo: String,
    pub file: String,
    pub name: String,
    pub kind: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub signature: Option<String>,
    pub docstring: Option<String>,
    #[serde(skip)]
    #[allow(dead_code)]
    pub embedding: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {
    pub id: Option<i64>,
    pub repo: String,
    pub file: String,
    pub method: String,
    pub path: String,
    pub handler: Option<String>,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub root_path: PathBuf,
    pub db_path: PathBuf,
    pub last_indexed: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_count: Option<usize>,
}

#[derive(Clone)]
pub struct Storage {
    pub base_dir: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".rustdex");
        std::fs::create_dir_all(&base_dir)?;

        let storage = Self { base_dir };
        
        // Ensure registry exists
        let conn = storage.get_registry_conn()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS repos (
                name         TEXT PRIMARY KEY,
                root_path    TEXT NOT NULL,
                db_path      TEXT NOT NULL UNIQUE,
                last_indexed DATETIME
            )",
            [],
        )?;

        Ok(storage)
    }

    pub fn get_registry_conn(&self) -> Result<Connection> {
        let registry_path = self.base_dir.join("registry.db");
        let conn = Connection::open(registry_path)?;
        let _ = conn.query_row("PRAGMA journal_mode=WAL", [], |_| Ok(()));
        Ok(conn)
    }

    pub fn get_db_path(&self, repo_name: &str) -> PathBuf {
        self.base_dir.join(format!("{}.db", repo_name.to_lowercase()))
    }

    pub fn open_repo_db(&self, db_path: &Path) -> Result<Connection> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        let _ = conn.query_row("PRAGMA journal_mode=WAL", [], |_| Ok(()));
        conn.execute("PRAGMA foreign_keys=ON", [])?;

        // Apply schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS symbols (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                repo        TEXT NOT NULL,
                file        TEXT NOT NULL,
                name        TEXT NOT NULL,
                kind        TEXT NOT NULL,
                start_byte  INTEGER NOT NULL,
                end_byte    INTEGER NOT NULL,
                signature   TEXT,
                docstring   TEXT,
                embedding   BLOB
            );
            CREATE INDEX IF NOT EXISTS idx_symbols_repo_name ON symbols(repo, name);
            CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
            CREATE INDEX IF NOT EXISTS idx_symbols_repo_file ON symbols(repo, file);
            CREATE INDEX IF NOT EXISTS idx_symbols_repo_kind ON symbols(repo, kind);

            CREATE TABLE IF NOT EXISTS edges (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                caller_id   INTEGER NOT NULL REFERENCES symbols(id) ON DELETE CASCADE,
                callee_name TEXT NOT NULL,
                callee_file TEXT,
                UNIQUE(caller_id, callee_name, callee_file)
            );
            CREATE INDEX IF NOT EXISTS idx_edges_caller ON edges(caller_id);
            CREATE INDEX IF NOT EXISTS idx_edges_callee ON edges(callee_name);

            CREATE TABLE IF NOT EXISTS files (
                repo        TEXT NOT NULL,
                path        TEXT NOT NULL,
                hash        TEXT NOT NULL,
                indexed_at  DATETIME NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (repo, path)
            );

            CREATE TABLE IF NOT EXISTS routes (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                repo        TEXT NOT NULL,
                file        TEXT NOT NULL,
                method      TEXT NOT NULL,
                path        TEXT NOT NULL,
                handler     TEXT,
                start_byte  INTEGER NOT NULL,
                end_byte    INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_routes_repo ON routes(repo);
            CREATE INDEX IF NOT EXISTS idx_routes_repo_path ON routes(repo, path);
            CREATE INDEX IF NOT EXISTS idx_routes_method ON routes(repo, method);"
        )?;

        Ok(conn)
    }

    pub fn register_repo(&self, info: &RepoInfo) -> Result<()> {
        let conn = self.get_registry_conn()?;
        conn.execute(
            "INSERT INTO repos (name, root_path, db_path, last_indexed)
             VALUES (?, ?, ?, datetime('now'))
             ON CONFLICT(name) DO UPDATE SET
                root_path = excluded.root_path,
                db_path = excluded.db_path,
                last_indexed = excluded.last_indexed",
            params![
                info.name.to_lowercase(),
                info.root_path.to_string_lossy(),
                info.db_path.to_string_lossy()
            ],
        )?;
        Ok(())
    }

    pub fn list_repos(&self) -> Result<Vec<RepoInfo>> {
        let conn = self.get_registry_conn()?;
        let mut stmt = conn.prepare("SELECT name, root_path, db_path, last_indexed FROM repos ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            let last_indexed: Option<String> = row.get(3)?;
            Ok(RepoInfo {
                name: row.get(0)?,
                root_path: PathBuf::from(row.get::<_, String>(1)?),
                db_path: PathBuf::from(row.get::<_, String>(2)?),
                last_indexed: last_indexed.and_then(|s| NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
                symbol_count: None,
            })
        })?;

        let mut repos = Vec::new();
        for repo in rows {
            repos.push(repo?);
        }
        Ok(repos)
    }

    pub fn get_repo_info(&self, name: &str) -> Result<Option<RepoInfo>> {
        let conn = self.get_registry_conn()?;
        let mut stmt = conn.prepare("SELECT name, root_path, db_path, last_indexed FROM repos WHERE name = ?")?;
        let mut rows = stmt.query_map(params![name.to_lowercase()], |row| {
            let last_indexed: Option<String> = row.get(3)?;
            Ok(RepoInfo {
                name: row.get(0)?,
                root_path: PathBuf::from(row.get::<_, String>(1)?),
                db_path: PathBuf::from(row.get::<_, String>(2)?),
                last_indexed: last_indexed.and_then(|s: String| NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
                symbol_count: None,
            })
        })?;

        if let Some(repo) = rows.next() {
            Ok(Some(repo?))
        } else {
            Ok(None)
        }
    }
}
