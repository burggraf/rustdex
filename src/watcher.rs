use anyhow::Result;
use notify::{Watcher, RecursiveMode, Event};
use std::path::Path;
use crate::storage::Storage;
use crate::indexer::Indexer;

pub fn watch_folder(path: &Path, storage: Storage) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::RecommendedWatcher::new(tx, notify::Config::default())?;
    watcher.watch(path, RecursiveMode::Recursive)?;

    let mut indexer = Indexer::new(storage)?;
    
    for res in rx {
        match res {
            Ok(event) => {
                if is_file_change(&event) {
                    println!("Change detected: {:?}", event.paths);
                    // In a real implementation, we would throttle and re-index only the changed files
                    // For now, just re-index the whole folder
                    let _ = indexer.index_folder(path, None);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn is_file_change(event: &Event) -> bool {
    event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove()
}
