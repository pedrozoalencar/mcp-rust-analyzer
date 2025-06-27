#!/bin/bash

# MCP Rust Analyzer Installation Script

set -e

echo "Installing MCP Rust Analyzer..."

# Check if rust-analyzer is installed
if ! command -v rust-analyzer &> /dev/null; then
    echo "rust-analyzer not found. Please install it first:"
    echo "  rustup component add rust-analyzer"
    echo "  or download from: https://github.com/rust-lang/rust-analyzer/releases"
    exit 1
fi

# Build the project
echo "Building MCP Rust Analyzer..."
cargo build --release

# Create installation directory
INSTALL_DIR="$HOME/.config/claude-code/mcp-servers"
mkdir -p "$INSTALL_DIR"

# Copy binary
echo "Installing binary..."
cp target/release/mcp-rust-analyzer "$INSTALL_DIR/"

# Copy configuration
echo "Installing configuration..."
cp mcp-config.json "$INSTALL_DIR/mcp-rust-analyzer.json"

# Update Claude Code configuration
CLAUDE_CONFIG="$HOME/.config/claude-code/config.json"
if [ -f "$CLAUDE_CONFIG" ]; then
    echo "Updating Claude Code configuration..."
    # Backup existing config
    cp "$CLAUDE_CONFIG" "$CLAUDE_CONFIG.bak"
    
    # Add MCP server configuration using jq if available
    if command -v jq &> /dev/null; then
        jq '.mcpServers."rust-analyzer" = {
            "command": "'$INSTALL_DIR'/mcp-rust-analyzer",
            "args": ["--project-path", "."],
            "env": {
                "RUST_LOG": "info",
                "USE_LSP": "true"
            }
        }' "$CLAUDE_CONFIG" > "$CLAUDE_CONFIG.tmp" && mv "$CLAUDE_CONFIG.tmp" "$CLAUDE_CONFIG"
    else
        echo "Please manually add the following to your Claude Code config:"
        echo '
  "mcpServers": {
    "rust-analyzer": {
      "command": "'$INSTALL_DIR'/mcp-rust-analyzer",
      "args": ["--project-path", "."],
      "env": {
        "RUST_LOG": "info",
        "USE_LSP": "true"
      }
    }
  }'
    fi
fi

echo "Installation complete!"
echo ""
echo "To use MCP Rust Analyzer in Claude Code:"
echo "1. Restart Claude Code"
echo "2. In a Rust project, use commands like:"
echo "   - mcp: get_hover src/main.rs:10:5"
echo "   - mcp: complete src/lib.rs:20:15"
echo "   - mcp: find_references MyStruct"