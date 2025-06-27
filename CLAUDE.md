# MCP Rust Analyzer - Claude Code Integration Guide

## Overview

This is an MCP (Model Context Protocol) server that provides rust-analyzer capabilities to Claude Code CLI. It enables intelligent Rust code analysis, refactoring, and navigation features.

## Key Features

- **Code Analysis**: Hover info, find references, symbol analysis
- **Auto-complete**: Context-aware completions and signature help  
- **Refactoring**: Rename symbols, extract functions, organize imports
- **Metrics**: Code statistics, dependency analysis, dead code detection

## Development Workflow

### Running Tests
Always run tests before committing:
```bash
cargo test
```

### Checking Code Quality
```bash
cargo clippy
cargo fmt --check
```

### Building
```bash
cargo build --release
```

## Architecture

The project consists of:
- `src/main.rs` - Entry point
- `src/server.rs` - MCP server implementation
- `src/analyzer.rs` - Rust analyzer integration
- `src/lsp_client.rs` - LSP client for rust-analyzer
- `src/commands/` - Command handlers for different features

## Testing Strategy

Follow TDD (Test-Driven Development):
1. Write tests first
2. Implement functionality
3. Ensure all tests pass
4. Commit with descriptive message

## LSP Integration

The server can operate in two modes:
- **Stub mode**: For testing without rust-analyzer
- **LSP mode**: Full integration with rust-analyzer (set USE_LSP=true)

## Important Commands

When implementing new features:
1. Always check existing patterns in the codebase
2. Add appropriate tests
3. Update documentation if needed
4. Run `cargo test` before committing

## Common Issues

- If LSP client fails to start, ensure rust-analyzer is installed
- For compilation errors, check that all dependencies are properly imported
- Use `RUST_LOG=debug` for detailed debugging information

## Next Steps

Priority areas for improvement:
1. Complete LSP integration for all commands
2. Add streaming diagnostics support
3. Implement workspace-wide symbol search
4. Add code action support for refactorings