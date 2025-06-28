# MCP Rust Analyzer

🦀 An intelligent Model Context Protocol (MCP) server that provides comprehensive Rust code analysis, refactoring, and navigation capabilities through rust-analyzer integration.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![MCP](https://img.shields.io/badge/MCP-Compatible-blue.svg)](https://modelcontextprotocol.io/)

## ✨ Features

### 🔍 **Code Analysis**
- **Hover Information**: Get detailed type information and documentation
- **Find References**: Locate all usage of symbols across your project  
- **Go to Implementation**: Navigate to trait implementations
- **Symbol Analysis**: Deep dive into symbol definitions and relationships

### ⚡ **Auto-completion & Navigation**
- **Smart Completions**: Context-aware code completions
- **Signature Help**: Function parameter assistance
- **Diagnostics**: Real-time error detection and suggestions

### 🔧 **Refactoring Tools**
- **Rename Symbol**: Safe rename across entire codebase
- **Extract Function**: Extract code into reusable functions
- **Organize Imports**: Clean up and optimize import statements

### 📊 **Project Metrics**
- **Code Statistics**: Lines of code, complexity metrics
- **Dependency Analysis**: Analyze Cargo.toml dependencies
- **Dead Code Detection**: Find unused code in your project
- **Project Structure**: Analyze module organization

### 🎯 **Smart Templates**
- **Code Snippets**: Pre-built templates for common patterns
- **Match Expressions**: Generate match arms automatically
- **Test Functions**: Quick test function templates

### 🚀 **Intelligent Operation Modes**
- **Claude Code CLI**: Automatic direct mode for maximum compatibility
- **Manual Terminal**: Smart daemon/client system for optimal performance
- **TTY Detection**: Automatically chooses the best mode based on context

## 🚀 Installation

### Prerequisites
- Rust toolchain (1.70+)
- `rust-analyzer` component: `rustup component add rust-analyzer`

### Install from Source
```bash
git clone https://github.com/pedrozoalencar/mcp-rust-analyzer.git
cd mcp-rust-analyzer
cargo build --release

# Add to PATH (permanent)
echo 'export PATH="/path/to/mcp-rust-analyzer/target/release:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## 📋 Usage

### With Claude Code CLI

1. **Add MCP Server**:
```bash
claude mcp add rust-analyzer mcp-rust-analyzer
```

2. **Verify Installation**:
```bash
claude mcp list
claude mcp test rust-analyzer
```

3. **Use in Any Rust Project**:
```bash
cd your-rust-project/
claude  # Automatically detects and analyzes Rust code!
```

### Manual Usage (Advanced)

#### **Daemon Mode** (Recommended for Manual Use)
```bash
# Start daemon for current project
mcp-rust-analyzer --daemon

# Check daemon status
mcp-rust-analyzer --status

# Stop daemon
mcp-rust-analyzer --stop

# Use client (auto-connects to daemon)
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | mcp-rust-analyzer
```

#### **Direct Mode**
```bash
# Direct stdin/stdout mode
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | mcp-rust-analyzer --server
```

## 🛠️ Architecture

### **Intelligent Mode Detection**
- **Claude Code CLI**: Automatically uses direct mode for maximum compatibility
- **Manual Terminal**: Uses smart daemon/client system for optimal performance
- **TTY Detection**: Automatically chooses the best mode based on context

### **Dual Operation Modes**

#### **Daemon/Client System** (Manual Use)
- **Background Daemon**: Persistent HTTP server per project
- **Auto-Port Selection**: Finds available ports automatically  
- **State Management**: Tracks daemons across multiple projects
- **Auto-Start**: Client automatically starts daemon if needed

#### **Direct Mode** (Claude Code CLI)
- **Zero Configuration**: Works out-of-the-box
- **LSP Integration**: Full rust-analyzer capabilities
- **Fast Startup**: Optimized for MCP protocol

### **LSP Integration**
- **Background Initialization**: Non-blocking rust-analyzer startup
- **Smart Caching**: Reuses LSP connections for performance
- **Error Handling**: Graceful fallbacks when LSP unavailable

## 📚 Available Tools

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