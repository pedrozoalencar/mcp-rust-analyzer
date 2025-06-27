# MCP Rust Analyzer

A Model Context Protocol (MCP) server that provides Rust language intelligence through integration with rust-analyzer. This server enables AI assistants like Claude to understand and work with Rust codebases with full IntelliSense capabilities.

## Features

### 🔍 Code Intelligence
- **Auto-completion**: Get context-aware code completions
- **Hover information**: View type information and documentation
- **Find references**: Locate all usages of symbols
- **Go to definition**: Navigate to symbol definitions
- **Signature help**: View function/method signatures while typing
- **Diagnostics**: Real-time error and warning detection

### 🛠️ Refactoring Tools
- **Rename**: Safely rename symbols across the project
- **Extract function**: Extract code into new functions
- **Inline**: Inline variables and functions
- **Organize imports**: Automatically organize use statements

### 📊 Code Analysis
- **Project structure**: Analyze module organization
- **Code metrics**: Lines of code, complexity, and more
- **Dependency analysis**: Visualize project dependencies
- **Dead code detection**: Find unused code
- **Performance suggestions**: Get optimization recommendations

### 🎯 MCP Features
- **Tools**: 14+ tools for code analysis and refactoring
- **Resources**: Access project structure, diagnostics, dependencies
- **Prompts**: Pre-configured prompts for common tasks
- **Full MCP Protocol**: Complete implementation of the MCP specification

## Installation

### Prerequisites
- Rust toolchain (rustc, cargo)
- rust-analyzer installed (`rustup component add rust-analyzer`)

### Build from source
```bash
git clone https://github.com/pedrozoalencar/mcp-rust-analyzer.git
cd mcp-rust-analyzer
cargo build --release
```

### Configure with Claude Code CLI

1. Add the MCP server:
```bash
claude mcp add rust-analyzer /path/to/mcp-rust-analyzer/target/release/mcp-rust-analyzer
```

2. Start Claude:
```bash
claude
```

## Usage

Once configured, the MCP server provides various tools that Claude can use:

### Example Commands
- "Analyze the project structure"
- "Show me code metrics for the src module"
- "Find all references to the `Parser` struct"
- "Get hover information for line 45 in main.rs"
- "Suggest completions at line 30, column 15"
- "Find unused code in the project"

### Available Tools

| Tool | Description |
|------|-------------|
| `project_structure` | Analyze project module organization |
| `code_metrics` | Get code statistics and metrics |
| `analyze_dependencies` | View dependency graph |
| `complete` | Get code completions at a position |
| `get_hover` | Get type/documentation info |
| `find_references` | Find all symbol references |
| `rename` | Rename symbols safely |
| `signature_help` | Get function signature help |
| `get_diagnostics` | Get compiler diagnostics |
| `analyze_symbol` | Analyze a symbol by name |
| `find_implementations` | Find trait implementations |
| `expand_snippet` | Expand code snippets |
| `find_dead_code` | Detect unused code |
| `suggest_improvements` | Get optimization suggestions |

### Resources

The server exposes these resources:
- `rust-analyzer://project/structure` - Project structure
- `rust-analyzer://project/diagnostics` - All diagnostics
- `rust-analyzer://project/dependencies` - Dependency graph
- `rust-analyzer://project/symbols` - Workspace symbols

### Prompts

Pre-configured prompts for common tasks:
- `analyze_code` - Comprehensive code analysis
- `refactor_code` - Guided refactoring
- `explain_error` - Error explanation with fixes
- `optimize_code` - Performance optimization

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Claude Code   │────▶│   MCP Server    │────▶│  rust-analyzer  │
│      (Host)     │◀────│  (This Project) │◀────│      (LSP)      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        JSON-RPC              Rust API               LSP Protocol
```

## Development

### Project Structure
```
src/
├── main.rs           # Entry point
├── server.rs         # MCP server implementation
├── analyzer.rs       # rust-analyzer integration
├── lsp_client.rs     # LSP client for rust-analyzer
└── commands/         # Command handlers
    ├── analysis.rs   # Code analysis commands
    ├── completion.rs # IntelliSense commands
    ├── metrics.rs    # Metrics and structure
    └── refactor.rs   # Refactoring commands
```

### Running Tests
```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test '*'

# Test MCP protocol compliance
./test_mcp_complete.sh
```

### Debug Mode
```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Test individual commands
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | cargo run
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Guidelines
1. Follow Rust naming conventions
2. Add tests for new features
3. Update documentation
4. Ensure all tests pass
5. Format code with `cargo fmt`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built on the [Model Context Protocol](https://modelcontextprotocol.io) specification
- Powered by [rust-analyzer](https://rust-analyzer.github.io/)
- Inspired by the need for better AI-assisted Rust development

---

Made with ❤️ for the Rust community