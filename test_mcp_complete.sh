#!/bin/bash

echo "=== TESTE COMPLETO DO PROTOCOLO MCP ==="
echo ""

MCP_PATH="/home/renato/workspaces/mcp-rust-analyzer/target/release/mcp-rust-analyzer"

test_mcp() {
    local name=$1
    local request=$2
    echo "----------------------------------------"
    echo "TESTE: $name"
    echo ""
    echo "$request" | $MCP_PATH 2>/dev/null | jq '.' || echo "$request" | $MCP_PATH 2>/dev/null
    echo ""
}

# 1. Initialize
test_mcp "1. INITIALIZE" \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'

# 2. List Tools
test_mcp "2. TOOLS/LIST" \
    '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

# 3. List Resources
test_mcp "3. RESOURCES/LIST" \
    '{"jsonrpc":"2.0","id":3,"method":"resources/list","params":{}}'

# 4. List Prompts
test_mcp "4. PROMPTS/LIST" \
    '{"jsonrpc":"2.0","id":4,"method":"prompts/list","params":{}}'

# 5. Read Resource
test_mcp "5. RESOURCES/READ - Project Structure" \
    '{"jsonrpc":"2.0","id":5,"method":"resources/read","params":{"uri":"rust-analyzer://project/structure"}}'

# 6. Get Prompt
test_mcp "6. PROMPTS/GET - Analyze Code" \
    '{"jsonrpc":"2.0","id":6,"method":"prompts/get","params":{"name":"analyze_code","arguments":{"file":"src/main.rs"}}}'

# 7. Call Tool
test_mcp "7. TOOLS/CALL - Project Structure" \
    '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"project_structure","arguments":{}}}'

# 8. Call Tool - Complete
test_mcp "8. TOOLS/CALL - Complete" \
    '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"complete","arguments":{"file":"src/main.rs","line":10,"column":5}}}'

# 9. Call Tool - Get Hover
test_mcp "9. TOOLS/CALL - Get Hover" \
    '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"get_hover","arguments":{"file":"src/main.rs","line":10,"column":5}}}'

# 10. Completion
test_mcp "10. COMPLETION/COMPLETE" \
    '{"jsonrpc":"2.0","id":10,"method":"completion/complete","params":{"ref":{"type":"ref/resource","uri":"rust-analyzer://project/structure"}}}'

echo "=== FIM DOS TESTES MCP ==="