#!/bin/bash

echo "=== TESTE SIMPLES DO MCP ==="
echo ""

# Teste 1: Project Structure
echo "1. TESTANDO PROJECT STRUCTURE:"
echo '{"jsonrpc":"2.0","id":1,"method":"project_structure","params":{"method":"project_structure"}}' | \
    /home/renato/workspaces/mcp-rust-analyzer/target/release/mcp-rust-analyzer 2>&1

echo ""
echo "----------------------------------------"
echo ""

# Teste 2: Code Metrics
echo "2. TESTANDO CODE METRICS:"
echo '{"jsonrpc":"2.0","id":2,"method":"code_metrics","params":{"method":"code_metrics","module":"src"}}' | \
    /home/renato/workspaces/mcp-rust-analyzer/target/release/mcp-rust-analyzer 2>&1

echo ""
echo "----------------------------------------"
echo ""

# Teste 3: Snippet
echo "3. TESTANDO EXPAND SNIPPET:"
echo '{"jsonrpc":"2.0","id":3,"method":"expand_snippet","params":{"method":"expand_snippet","name":"for_loop"}}' | \
    /home/renato/workspaces/mcp-rust-analyzer/target/release/mcp-rust-analyzer 2>&1

echo ""
echo "----------------------------------------"
echo ""

# Teste 4: Erro
echo "4. TESTANDO MÉTODO INVÁLIDO (deve retornar erro):"
echo '{"jsonrpc":"2.0","id":4,"method":"metodo_invalido","params":{}}' | \
    /home/renato/workspaces/mcp-rust-analyzer/target/release/mcp-rust-analyzer 2>&1

echo ""
echo "=== FIM DOS TESTES ===