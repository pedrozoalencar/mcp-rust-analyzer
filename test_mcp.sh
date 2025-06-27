#!/bin/bash

echo "Testing MCP Rust Analyzer..."

# Test project structure
echo "1. Testing project_structure command:"
echo '{"jsonrpc":"2.0","id":1,"method":"project_structure","params":{"method":"project_structure"}}' | cargo run --quiet 2>/dev/null

echo -e "\n2. Testing code_metrics command:"
echo '{"jsonrpc":"2.0","id":2,"method":"code_metrics","params":{"method":"code_metrics","module":"src"}}' | cargo run --quiet 2>/dev/null

echo -e "\n3. Testing analyze_dependencies command:"
echo '{"jsonrpc":"2.0","id":3,"method":"analyze_dependencies","params":{"method":"analyze_dependencies"}}' | cargo run --quiet 2>/dev/null

echo -e "\n4. Testing get_hover command:"
echo '{"jsonrpc":"2.0","id":4,"method":"get_hover","params":{"method":"get_hover","file":"examples/demo.rs","line":11,"column":9}}' | cargo run --quiet 2>/dev/null

echo -e "\n5. Testing complete command:"
echo '{"jsonrpc":"2.0","id":5,"method":"complete","params":{"method":"complete","file":"examples/demo.rs","line":7,"column":20}}' | cargo run --quiet 2>/dev/null

echo -e "\n6. Testing expand_snippet command:"
echo '{"jsonrpc":"2.0","id":6,"method":"expand_snippet","params":{"method":"expand_snippet","name":"match_expr"}}' | cargo run --quiet 2>/dev/null