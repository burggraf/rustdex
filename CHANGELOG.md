# Changelog

## 0.4.1 - 2026-03-12

### Fixed
- Fixed npm installer - now correctly downloads release files with "v" prefix (e.g., rustdex-v0.4.1-darwin-arm64.zip)
- GitHub release workflow creates files with "v" prefix, which installer now expects

## 0.4.0 - 2026-03-12

### Added
- npm package distribution - users can now install with `npm install -g rustdex`
- Automatic platform detection and binary download during npm installation
- Cross-platform binary support: macOS (ARM64/x64), Linux (ARM64/AMD64), Windows (ARM64/AMD64)
- Automated binary packaging via GitHub Actions with code-signed macOS binaries
- npm uninstall cleanup script
- Interactive publishing helper script (`publish-npm.sh`)

### Changed
- Updated installation documentation with npm as the recommended method
- Improved user experience for easy installation across all platforms

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
