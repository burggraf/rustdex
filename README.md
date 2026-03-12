# RustDex User Documentation

RustDex is a high-performance, universal code-indexer and semantic search tool built in Rust. It allows AI agents and developers to navigate large codebases instantly by providing exact symbol locations and natural language search capabilities, all while running 100% locally.

## Acknowledgments

This project is based on the excellent work at [SymDex](https://github.com/husnainpk/SymDex). While RustDex is a complete rewrite in Rust, it builds upon the core concepts and ideas pioneered by SymDex.

## Features

- **Symbol Indexing**: Extract functions, classes, and methods with exact byte offsets using Tree-sitter.
- **Semantic Search**: Find code by "what it does" using local BERT embeddings (no API keys required).
- **HTTP Route Extraction**: Automatically identifies API endpoints in Web frameworks.
- **Cross-Repo Search**: Query across all your indexed projects from a single interface.
- **Watch Mode**: Automatically re-indexes files as you save them.
- **Agent Optimized**: Provides structured JSON output for easy integration with AI coding harnesses.

## Supported Languages

RustDex supports a wide range of programming languages:
- **Rust**, **Python**, **JavaScript**, **TypeScript (TSX)**
- **Go**, **Java**, **PHP**, **C**, **C++**
- **Elixir**, **Ruby**, **Vue**

---

## Installation

### npm (Recommended - Easiest)
The easiest way to install RustDex on any platform:

```bash
npm install -g rustdex
```

This automatically downloads the appropriate binary for your platform (macOS, Linux, or Windows) and installs it to your PATH. Supports:
- macOS: Apple Silicon (ARM64) and Intel (x64)
- Linux: ARM64 and AMD64
- Windows: ARM64 and AMD64

### From Source
1. Ensure you have the [Rust toolchain](https://rustup.rs/) installed.
2. Clone the repository and build:
   ```bash
   cd ~/dev/rustdex
   cargo build --release
   ```
3. Move the binary to your PATH:
   ```bash
   cp target/release/rustdex /usr/local/bin/
   ```

### Cargo Install
If you already have Rust installed:

```bash
cargo install rustdex --locked
```

---

## Getting Started

### 1. Indexing a Project
To start using RustDex, you first need to index your codebase. This creates a local SQLite database containing symbol metadata and vector embeddings.

```bash
# Basic indexing
rustdex index /path/to/your/project --name my-project

# Get JSON output with symbol count
rustdex index /path/to/your/project --name my-project --json
```
*Note: If `--name` is omitted, the folder name will be used.*

**JSON Output Example:**
```json
{
  "name": "my-project",
  "root_path": "/path/to/your/project",
  "db_path": "~/.rustdex/my-project.db",
  "last_indexed": "2026-03-11T16:45:20.126182",
  "symbol_count": 1543
}
```

### 2. Searching for Symbols
Locate a function or class by its name across the repo.

```bash
# Text output
rustdex search "validate_user" --repo my-project

# JSON output
rustdex search "validate_user" --repo my-project --json
```

### 3. Semantic Search (Natural Language)
If you don't know the exact function name, you can search using natural language.

```bash
# Text output
rustdex semantic "how do we handle password hashing" --repo my-project

# JSON output
rustdex semantic "how do we handle password hashing" --repo my-project --json
```

### 4. Exploring API Routes
Find all HTTP routes defined in your project (supports Flask, FastAPI, Django, and Express).

```bash
# All routes (text)
rustdex routes my-project

# POST routes only (JSON)
rustdex routes my-project --method POST --json
```

### 5. Keeping the Index Fresh
Run the watch command in a terminal to automatically update the index whenever you save a file.

```bash
rustdex watch /path/to/your/project
```

### 6. Listing Repositories
View all your indexed repositories.

```bash
# Text output
rustdex list-repos

# JSON output
rustdex list-repos --json
```

---

## Pi Extension (pi-rustdex)

For the best experience with Pi, install the official Pi extension:

```bash
pi install npm:pi-rustdex
```

### Features

The Pi extension provides these tools:

| Tool | Description |
|------|-------------|
| `rustdex_index` | Index a codebase for searching |
| `rustdex_search` | Find symbols by exact name |
| `rustdex_semantic` | Natural language code search |
| `rustdex_routes` | Extract HTTP API endpoints |
| `rustdex_list_repos` | List all indexed repositories |
| `rustdex_read_symbol` | Read source code by byte range |

### Example Usage in Pi

```
Index my project at /home/user/webapp

Find the validateToken function in webapp

Search for "user authentication logic" in webapp using semantic search

Show me all POST routes in webapp
```

---

## Advanced Usage

### Using with AI Agents (Pi, Claude Code, Cursor)
RustDex is designed to be called by AI agents to reduce token consumption. Instead of the agent reading full files, it calls RustDex to find the exact byte range of a symbol.

**JSON Output:**
Add the `--json` flag to any command to get machine-readable output:
```bash
# Index with JSON output
rustdex index /path/to/project --name my-project --json

# Search with JSON output
rustdex search "AuthStore" --repo my-project --json

# List repos with JSON output
rustdex list-repos --json

# Routes with JSON output
rustdex routes my-project --json
```

### Management
- **List all indexed repos**: `rustdex list-repos` or `rustdex list-repos --json`
- **Check version**: `rustdex --version` or `rustdex -V`
- **Storage Location**: All data is stored in `~/.rustdex/`.
  - `registry.db`: Tracks all projects and their paths.
  - `<repo_name>.db`: Contains the actual index for each project.

### Excluded Directories
The indexer automatically skips common directories that don't contain source code:

| Category | Excluded Directories |
|----------|---------------------|
| Version Control | `.git`, `.svn`, `.hg`, `.bzr` |
| Package Managers | `node_modules`, `bower_components`, `jspm_packages` |
| Python | `__pycache__`, `venv`, `.venv`, `.tox`, `.eggs`, `.mypy_cache`, `.pytest_cache`, `.ruff_cache` |
| Rust/Go/PHP | `target`, `vendor`, `third_party`, `third-party` |
| Build Outputs | `dist`, `build` |
| Git Worktrees | `.worktree` |

**Excluded file extensions:** `.pyc`, `.so`, `.dylib`, `.dll`, `.exe`, `.bin`, `.png`, `.jpg`, `.db`, `.sqlite`

---

## Changelog

### v0.4.1 (Latest)
- Fixed npm installer to correctly download release files with "v" prefix
- npm package distribution working correctly

### v0.4.0
- npm package distribution - install with `npm install -g rustdex`
- Automatic platform detection and binary download
- Cross-platform binary support: macOS, Linux, Windows
- Automated binary packaging via GitHub Actions
- Interactive publishing helper script

### v0.3.0
- Expanded list of excluded directories for indexing:
  - **Version control**: `.git`, `.svn`, `.hg`, `.bzr`
  - **Package managers**: `node_modules`, `bower_components`, `jspm_packages`
  - **Python environments**: `__pycache__`, `venv`, `.venv`, `.tox`, `.eggs`, `.mypy_cache`, `.pytest_cache`, `.ruff_cache`
  - **Go/PHP vendor**: `vendor`, `third_party`, `third-party`
  - **Build outputs**: `dist`, `build`, `target`
  - **Git worktrees**: `.worktree`
- File extensions excluded: `.pyc`, `.so`, `.dylib`, `.dll`, `.exe`, `.bin`, `.png`, `.jpg`, `.db`, `.sqlite`

### v0.2.0
- Added `--version` / `-V` flag to display version information
- Added `--json` support to `index` and `list-repos` commands
- Added `symbol_count` field to indexing output
- Progress output suppressed when `--json` flag is used
- Improved JSON consistency across all commands

### v0.1.0
- Initial release with symbol indexing, semantic search, and route extraction

---

## Troubleshooting

### Binary Size
If you need to reduce the binary size for distribution, ensure you build with the release profile:
```bash
cargo build --release
```
Current builds are optimized using LTO (Link Time Optimization) and symbol stripping to keep the footprint minimal despite containing full ML models.

### macOS Security (Killed: 9)
Do not use `upx` on Apple Silicon (M1/M2/M3) versions of RustDex. The binary requires strict memory alignment that `upx` breaks. The standard `cargo build --release` binary is the recommended version for macOS.
