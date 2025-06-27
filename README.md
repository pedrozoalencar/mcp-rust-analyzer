# MCP Rust Analyzer

Um servidor MCP (Model Context Protocol) que integra o rust-analyzer para fornecer análise avançada de código Rust ao Claude Code CLI.

## Funcionalidades

### Análise de Código
- **Análise de símbolos**: Encontre definições, referências e implementações
- **Diagnósticos**: Obtenha erros e avisos do compilador em tempo real
- **Hover information**: Informações detalhadas sobre tipos e documentação
- **Navegação de código**: Go to definition, find all references
- **Análise de dependências**: Visualize a árvore de dependências do projeto

### Auto-complete e IntelliSense
- **Auto-complete contextual**: Sugestões inteligentes baseadas no contexto
- **Assinatura de métodos**: Visualize parâmetros e tipos durante a digitação
- **Completion de imports**: Sugere e auto-importa módulos necessários
- **Snippet expansion**: Templates de código para estruturas comuns
- **Type inference**: Sugestões baseadas em inferência de tipos
- **Trait completion**: Sugestões de métodos disponíveis para traits
- **Lifetime suggestions**: Ajuda com lifetimes e borrowing

### Refatorações
- **Renomear símbolos**: Renomeie variáveis, funções, tipos com segurança
- **Extract function/method**: Extraia código selecionado para nova função
- **Inline variable/function**: Inline de variáveis e funções
- **Change signature**: Modifique assinaturas de funções
- **Move item**: Mova items entre módulos

### Informações Estratégicas
- **Métricas de código**: Complexidade ciclomática, linhas de código
- **Análise de arquitetura**: Visualize a estrutura de módulos
- **Detecção de code smells**: Identifique problemas comuns
- **Análise de performance**: Sugestões de otimização
- **Cobertura de testes**: Identifique código não testado

## Instalação

```bash
# Clone o repositório
git clone https://github.com/seu-usuario/mcp-rust-analyzer
cd mcp-rust-analyzer

# Compile o projeto
cargo build --release

# O binário estará em target/release/mcp-rust-analyzer
```

## Configuração para Claude Code CLI

Adicione ao seu arquivo de configuração do Claude Code:

```json
{
  "mcpServers": {
    "rust-analyzer": {
      "command": "/caminho/para/mcp-rust-analyzer",
      "args": ["--project-path", "."],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## Uso

### Comandos Disponíveis

#### Análise
- `analyze_symbol <nome>` - Analisa um símbolo específico
- `find_references <nome>` - Encontra todas as referências
- `get_diagnostics [arquivo]` - Obtém diagnósticos do compilador
- `get_hover <arquivo:linha:coluna>` - Informações sobre posição
- `find_implementations <trait>` - Lista implementações de trait

#### Auto-complete e Assinaturas
- `complete <arquivo:linha:coluna>` - Obter sugestões de auto-complete
- `signature_help <arquivo:linha:coluna>` - Assinatura de método/função
- `get_completions <contexto>` - Lista completions para contexto
- `resolve_import <símbolo>` - Sugere imports para símbolo
- `expand_snippet <nome>` - Expande snippet de código

#### Refatoração
- `rename <nome_antigo> <nome_novo>` - Renomeia símbolo
- `extract_function <arquivo:inicio:fim> <nome>` - Extrai função
- `inline <nome>` - Inline de variável/função
- `organize_imports <arquivo>` - Organiza imports

#### Análise Estratégica
- `project_structure` - Visualiza estrutura do projeto
- `analyze_dependencies` - Analisa dependências
- `code_metrics [módulo]` - Métricas de código
- `find_dead_code` - Encontra código não utilizado
- `suggest_improvements <arquivo>` - Sugestões de melhorias

### Exemplos de Uso

```bash
# Analisar estrutura do projeto
mcp: project_structure

# Encontrar todas as referências de uma função
mcp: find_references my_function

# Renomear uma variável em todo o projeto
mcp: rename old_name new_name

# Obter métricas de um módulo
mcp: code_metrics src/lib.rs

# Sugerir melhorias para um arquivo
mcp: suggest_improvements src/main.rs

# Obter auto-complete em uma posição
mcp: complete src/main.rs:45:20

# Ver assinatura de método sendo chamado
mcp: signature_help src/lib.rs:100:15

# Expandir um snippet
mcp: expand_snippet match_expr
```

## Arquitetura

O MCP Rust Analyzer é construído sobre:
- **rust-analyzer**: Motor de análise de código Rust
- **Tower LSP**: Framework para Language Server Protocol
- **MCP Protocol**: Protocolo de comunicação com Claude

### Componentes Principais

1. **Servidor MCP**: Gerencia comunicação com Claude Code
2. **Analisador Rust**: Integração com rust-analyzer
3. **Motor de Refatoração**: Implementa refatorações seguras
4. **Analisador Estratégico**: Fornece insights de alto nível

## Desenvolvimento

### Estrutura do Projeto

```
mcp-rust-analyzer/
├── src/
│   ├── main.rs          # Entrada do servidor MCP
│   ├── server.rs        # Implementação do servidor
│   ├── analyzer.rs      # Integração rust-analyzer
│   ├── refactor.rs      # Motor de refatoração
│   ├── metrics.rs       # Análise de métricas
│   └── commands/        # Implementação dos comandos
├── tests/               # Testes de integração
├── Cargo.toml
└── README.md
```

### Contribuindo

1. Fork o projeto
2. Crie uma branch para sua feature
3. Commit suas mudanças
4. Push para a branch
5. Abra um Pull Request

## Licença

MIT