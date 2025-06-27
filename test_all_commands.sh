#!/bin/bash

echo "=== TESTE COMPLETO DO MCP RUST ANALYZER ==="
echo ""

MCP_PATH="/home/renato/workspaces/mcp-rust-analyzer/target/release/mcp-rust-analyzer"

# Função para testar comando
test_command() {
    local name=$1
    local request=$2
    echo "----------------------------------------"
    echo "TESTANDO: $name"
    echo "REQUEST: $request"
    echo ""
    echo "RESPONSE:"
    echo "$request" | $MCP_PATH 2>/dev/null | jq '.'
    echo ""
}

# 1. Teste project_structure
test_command "project_structure" \
    '{"jsonrpc":"2.0","id":1,"method":"project_structure","params":{"method":"project_structure"}}'

# 2. Teste code_metrics
test_command "code_metrics" \
    '{"jsonrpc":"2.0","id":2,"method":"code_metrics","params":{"method":"code_metrics","module":"src"}}'

# 3. Teste analyze_dependencies
test_command "analyze_dependencies" \
    '{"jsonrpc":"2.0","id":3,"method":"analyze_dependencies","params":{"method":"analyze_dependencies"}}'

# 4. Teste expand_snippet
test_command "expand_snippet" \
    '{"jsonrpc":"2.0","id":4,"method":"expand_snippet","params":{"method":"expand_snippet","name":"match_expr"}}'

# 5. Teste get_hover
test_command "get_hover" \
    '{"jsonrpc":"2.0","id":5,"method":"get_hover","params":{"method":"get_hover","file":"src/main.rs","line":10,"column":5}}'

# 6. Teste complete
test_command "complete" \
    '{"jsonrpc":"2.0","id":6,"method":"complete","params":{"method":"complete","file":"src/main.rs","line":15,"column":10}}'

# 7. Teste find_references
test_command "find_references" \
    '{"jsonrpc":"2.0","id":7,"method":"find_references","params":{"method":"find_references","file":"src/analyzer.rs","line":50,"column":15}}'

# 8. Teste método inválido (deve retornar erro)
test_command "método_inválido" \
    '{"jsonrpc":"2.0","id":8,"method":"invalid_method","params":{}}'

echo "=== FIM DOS TESTES ==="