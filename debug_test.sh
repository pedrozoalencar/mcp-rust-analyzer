#!/bin/bash

echo "Testing MCP server..."

# Test with debug logging enabled
export RUST_LOG=debug

# Send request and capture both stdout and stderr
REQUEST='{"jsonrpc":"2.0","id":1,"method":"project_structure","params":{"method":"project_structure"}}'
echo "Sending request: $REQUEST"
echo ""

echo "$REQUEST" | /home/renato/workspaces/mcp-rust-analyzer/target/release/mcp-rust-analyzer 2>&1 | tee output.log

echo ""
echo "Output saved to output.log"