# MCP Rust Analyzer - Test Suite

Este diretório contém os testes de integração completos para o servidor MCP.

## Testes Implementados

### 1. `mcp_direct_tests.rs`
Testes diretos que instanciam o servidor MCP e validam cada comando:

- ✅ **test_project_structure** - Valida estrutura do projeto
- ✅ **test_code_metrics** - Verifica métricas de código
- ✅ **test_analyze_dependencies** - Testa análise de dependências
- ✅ **test_expand_snippet** - Valida expansão de snippets
- ✅ **test_analyze_symbol** - Testa análise de símbolos
- ✅ **test_get_hover** - Verifica informações hover
- ✅ **test_complete** - Testa auto-complete
- ✅ **test_find_references** - Valida busca de referências
- ✅ **test_error_handling** - Testa tratamento de erros
- ✅ **test_missing_params** - Valida parâmetros obrigatórios
- ✅ **test_response_format_consistency** - Verifica consistência
- ✅ **test_edge_cases** - Testa casos extremos

### 2. `mcp_validation_tests.rs`
Testes de validação detalhada dos formatos de resposta:

- ✅ **validate_project_structure_response** - Formato completo da estrutura
- ✅ **validate_code_metrics_response** - Consistência numérica das métricas
- ✅ **validate_dependencies_response** - Formato das dependências
- ✅ **validate_snippet_expansion** - Conteúdo dos snippets
- ✅ **validate_error_responses** - Formato de erros JSON-RPC
- ✅ **validate_position_handling** - Tratamento de posições no código
- ✅ **validate_file_path_handling** - Manipulação de caminhos
- ✅ **validate_large_response_handling** - Respostas grandes

### 3. `mcp_response_format_tests.rs` (não executado)
Testes de formato usando processo externo (para referência futura).

### 4. `mcp_edge_case_tests.rs` (não executado)
Casos extremos com processo externo (para referência futura).

## Executando os Testes

```bash
# Executar todos os testes MCP
cargo test --test mcp_direct_tests --test mcp_validation_tests

# Executar teste específico
cargo test test_project_structure

# Executar com output detalhado
cargo test -- --nocapture
```

## Cobertura dos Testes

### Comandos Testados
- ✅ project_structure
- ✅ code_metrics
- ✅ analyze_dependencies
- ✅ expand_snippet
- ✅ analyze_symbol
- ✅ get_hover
- ✅ complete
- ✅ find_references
- ✅ get_diagnostics
- ✅ signature_help
- ✅ rename
- ✅ find_dead_code
- ✅ suggest_improvements

### Validações Realizadas
1. **Formato JSON-RPC**: Todos os responses seguem o padrão
2. **Campos Obrigatórios**: Validação de campos required
3. **Tipos de Dados**: Verificação de tipos corretos
4. **Consistência**: Dados retornados são logicamente consistentes
5. **Tratamento de Erros**: Erros seguem padrão JSON-RPC
6. **Casos Extremos**: Strings vazias, números grandes, etc.

## Resultados

Todos os 20 testes passam com sucesso, validando:
- Funcionalidade completa do servidor MCP
- Formato correto de todas as respostas
- Tratamento adequado de erros
- Robustez com entradas inválidas