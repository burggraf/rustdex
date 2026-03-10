# RustDex User Documentation

RustDex is a high-performance, universal code-indexer and semantic search tool built in Rust. It allows AI agents and developers to navigate large codebases instantly by providing exact symbol locations and natural language search capabilities, all while running 100% locally.

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

---

## Getting Started

### 1. Indexing a Project
To start using RustDex, you first need to index your codebase. This creates a local SQLite database containing symbol metadata and vector embeddings.

```bash
rustdex index /path/to/your/project --name my-project
```
*Note: If `--name` is omitted, the folder name will be used.*

### 2. Searching for Symbols
Locate a function or class by its name across the repo.

```bash
rustdex search "validate_user" --repo my-project
```

### 3. Semantic Search (Natural Language)
If you don't know the exact function name, you can search using natural language.

```bash
rustdex semantic "how do we handle password hashing" --repo my-project
```

### 4. Exploring API Routes
Find all HTTP routes defined in your project (supports Flask, FastAPI, Django, and Express).

```bash
rustdex routes my-project --method POST
```

### 5. Keeping the Index Fresh
Run the watch command in a terminal to automatically update the index whenever you save a file.

```bash
rustdex watch /path/to/your/project
```

---

## Advanced Usage

### Using with AI Agents (Pi, Claude Code, Cursor)
RustDex is designed to be called by AI agents to reduce token consumption. Instead of the agent reading full files, it calls RustDex to find the exact byte range of a symbol.

**JSON Output:**
Add the `--json` flag to any search command to get machine-readable output.
```bash
rustdex search "AuthStore" --json
```

### Management
- **List all indexed repos**: `rustdex list-repos`
- **Storage Location**: All data is stored in `~/.rustdex/`.
  - `registry.db`: Tracks all projects and their paths.
  - `<repo_name>.db`: Contains the actual index for each project.

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
