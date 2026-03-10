use clap::{Parser, Subcommand};
use anyhow::Result;

mod storage;
mod indexer;
mod parser;
mod search;
mod route_extractor;
mod embedding;
mod watcher;

// mod mcp;

#[derive(Parser)]
#[command(name = "rustdex")]
#[command(about = "Universal code-indexer for AI agents (Rust version)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Index a folder and its subdirectories
    Index {
        /// Path to the folder to index
        path: String,
        /// Optional name for the repo (defaults to folder name)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Search for symbols by name
    Search {
        /// The symbol name to search for
        query: String,
        /// Optional repo name to limit search
        #[arg(short, long)]
        repo: Option<String>,
        /// Optional kind to filter (e.g., function, class)
        #[arg(short, long)]
        kind: Option<String>,
        /// Max number of results (default 20)
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
    /// Semantic search using natural language
    Semantic {
        /// Natural language query
        query: String,
        /// Optional repo name
        #[arg(short, long)]
        repo: Option<String>,
        /// Max number of results (default 10)
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
    /// Search for HTTP routes
    Routes {
        /// Repo name
        repo: String,
        /// Optional method (e.g., GET, POST)
        #[arg(short, long)]
        method: Option<String>,
        /// Optional path substring
        #[arg(short, long)]
        path: Option<String>,
        /// Max results (default 50)
        #[arg(short, long, default_value_t = 50)]
        limit: usize,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
    /// Watch a folder for changes and auto-reindex
    Watch {
        /// Path to the folder
        path: String,
    },
    /// List all indexed repositories
    ListRepos,
    // Mcp,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let storage = storage::Storage::new()?;

    match cli.command {
        Commands::Index { path, name } => {
            let mut indexer = indexer::Indexer::new(storage)?;
            let info = indexer.index_folder(std::path::Path::new(&path), name)?;
            println!("Finished indexing repo: {}", info.name);
        }
        Commands::Search { query, repo, kind, limit, json } => {
            let searcher = search::Searcher::new(storage)?;
            let results = searcher.search_symbols(&query, repo.as_deref(), kind.as_deref(), limit)?;
            if json {
                println!("{}", serde_json::to_string(&results)?);
            } else {
                for sym in results {
                    println!(
                        "{} [{}] - {} (bytes {}-{})",
                        sym.name,
                        sym.kind,
                        sym.file,
                        sym.start_byte,
                        sym.end_byte
                    );
                }
            }
        }
        Commands::Semantic { query, repo, limit, json } => {
            let mut searcher = search::Searcher::new(storage)?;
            let results = searcher.search_semantic(&query, repo.as_deref(), limit)?;
            if json {
                // Map to a list of (Symbol, score) for easier parsing
                let json_results: Vec<serde_json::Value> = results.into_iter().map(|(s, score)| {
                    let mut v = serde_json::to_value(&s).unwrap();
                    if let Some(obj) = v.as_object_mut() {
                        obj.insert("score".to_string(), serde_json::Value::from(score));
                    }
                    v
                }).collect();
                println!("{}", serde_json::to_string(&json_results)?);
            } else {
                for (sym, score) in results {
                    println!(
                        "[{:.4}] {} [{}] - {}",
                        score, sym.name, sym.kind, sym.file
                    );
                }
            }
        }
        Commands::Routes { repo, method, path, limit, json } => {
            let searcher = search::Searcher::new(storage)?;
            let results = searcher.search_routes(&repo, method.as_deref(), path.as_deref(), limit)?;
            if json {
                println!("{}", serde_json::to_string(&results)?);
            } else {
                for route in results {
                    println!(
                        "{} {} -> {} ({})",
                        route.method,
                        route.path,
                        route.handler.unwrap_or_default(),
                        route.file
                    );
                }
            }
        }
        Commands::Watch { path } => {
            println!("Watching {} for changes...", path);
            watcher::watch_folder(std::path::Path::new(&path), storage)?;
        }
        Commands::ListRepos => {
            let repos = storage.list_repos()?;
            for repo in repos {
                println!(
                    "{} - {} (indexed: {:?})",
                    repo.name,
                    repo.root_path.display(),
                    repo.last_indexed
                );
            }
        }
    }

    Ok(())
}
