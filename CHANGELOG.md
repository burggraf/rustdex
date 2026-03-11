# Changelog

## 0.3.0 - 2026-03-11

### Added
- Expanded list of excluded directories for indexing:
  - **Version control**: `.git`, `.svn`, `.hg`, `.bzr`
  - **Package managers**: `node_modules`, `bower_components`, `jspm_packages`
  - **Python environments**: `__pycache__`, `venv`, `.venv`, `.tox`, `.eggs`, `.mypy_cache`, `.pytest_cache`, `.ruff_cache`
  - **Go/PHP vendor**: `vendor`, `third_party`, `third-party`
  - **Build outputs**: `dist`, `build`, `target`
  - **Git worktrees**: `.worktree`
- File extensions excluded: `.pyc`, `.so`, `.dylib`, `.dll`, `.exe`, `.bin`, `.png`, `.jpg`, `.db`, `.sqlite`

### Changed
- Indexer now skips common cache directories and build artifacts automatically

## 0.2.0 - 2026-03-11

### Added
- `--version` / `-V` flag to CLI - displays version information
- `--json` flag to `index` command - outputs indexing results as JSON
- `--json` flag to `list-repos` command - outputs repository list as JSON
- `symbol_count` field in `RepoInfo` - returns the number of symbols indexed

### Changed
- Progress output ("Indexing...", "Loading embedding engine...", "Indexed X symbols.") is now suppressed when `--json` flag is used
- Text output remains the default for backward compatibility

### Fixed
- JSON output is now consistent across all commands that support it

## 0.1.0 - Initial Release

### Features
- Index codebases with SQLite storage
- Symbol search by name
- Semantic search using natural language
- HTTP route extraction
- File watching for auto-reindex
- Repository listing
