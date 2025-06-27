# 📈 Plano de Melhoria do MCP Rust Analyzer

## 🎯 Objetivo
Transformar todas as ferramentas do MCP em recursos de alta produtividade, maximizando a integração com rust-analyzer.

## 📊 Status Atual

### ✅ Ferramentas Funcionais (4/14)
- `project_structure` - ⭐⭐⭐⭐⭐
- `code_metrics` - ⭐⭐⭐⭐⭐
- `analyze_dependencies` - ⭐⭐⭐⭐
- `expand_snippet` - ⭐⭐⭐

### 🔧 Ferramentas com Problemas (10/14)
- `rename` - Erro de parâmetros
- `get_hover` - Retorna null
- `complete` - Retorna vazio
- `find_references` - Retorna vazio
- `signature_help` - Não implementado
- `find_implementations` - Pendente
- `get_diagnostics` - Pendente
- `analyze_symbol` - Pendente
- `find_dead_code` - Análise pendente
- `suggest_improvements` - Implementação básica

## 🛠️ Plano de Implementação

### Fase 1: Correções Críticas (1-2 dias)

#### 1.1 Corrigir `rename` ✅
- [x] Ajustar parâmetros para usar posição
- [ ] Implementar método `rename` no analyzer
- [ ] Testar com casos reais

#### 1.2 Ativar USE_LSP por padrão
```rust
// analyzer.rs
let use_lsp = env::var("USE_LSP")
    .unwrap_or_else(|_| "true".to_string())  // Mudar default para true
    .parse::<bool>()
    .unwrap_or(true);
```

#### 1.3 Melhorar inicialização do LSP
- [ ] Adicionar retry logic
- [ ] Melhor tratamento de erros
- [ ] Logs mais informativos

### Fase 2: Implementar Ferramentas LSP (3-5 dias)

#### 2.1 Completar métodos no LspClient
```rust
// Adicionar em lsp_client.rs
pub async fn signature_help(&mut self, params: Value) -> Result<Value>
pub async fn find_implementations(&mut self, params: Value) -> Result<Value>
pub async fn diagnostics(&mut self, params: Value) -> Result<Value>
pub async fn code_action(&mut self, params: Value) -> Result<Value>
```

#### 2.2 Implementar handlers corretos
- [ ] `get_hover` - Formatar resposta LSP corretamente
- [ ] `complete` - Processar items de completion
- [ ] `find_references` - Converter locations LSP
- [ ] `signature_help` - Extrair assinaturas
- [ ] `find_implementations` - Buscar implementações

#### 2.3 Adicionar cache de documentos
```rust
struct DocumentCache {
    documents: HashMap<String, Document>,
    version: HashMap<String, i32>,
}
```

### Fase 3: Ferramentas Avançadas (5-7 dias)

#### 3.1 `analyze_symbol` inteligente
- [ ] Busca global de símbolos
- [ ] Análise de tipo e traits
- [ ] Grafo de dependências do símbolo

#### 3.2 `find_dead_code` real
- [ ] Integrar com `cargo check --all-targets`
- [ ] Parser de warnings
- [ ] Análise de uso

#### 3.3 `suggest_improvements`
- [ ] Integrar com clippy
- [ ] Sugestões de performance
- [ ] Padrões idiomáticos

### Fase 4: Otimizações (2-3 dias)

#### 4.1 Performance
- [ ] Pool de conexões LSP
- [ ] Cache de resultados
- [ ] Processamento assíncrono

#### 4.2 Robustez
- [ ] Testes de integração completos
- [ ] Tratamento de edge cases
- [ ] Recuperação de falhas

## 📋 Tarefas Imediatas

1. **Corrigir rename** (30 min) ✅
2. **Ativar LSP por padrão** (15 min)
3. **Melhorar logs de debug** (30 min)
4. **Implementar signature_help** (2h)
5. **Corrigir get_hover formatação** (1h)
6. **Implementar complete parsing** (2h)
7. **Adicionar testes reais** (3h)

## 🎯 Métricas de Sucesso

- [ ] 100% das ferramentas retornando dados válidos
- [ ] Tempo de resposta < 500ms para todas as operações
- [ ] Zero erros em operações normais
- [ ] Cobertura de testes > 80%

## 🚀 Benefícios Esperados

### Ganhos de Produtividade
- **IntelliSense completo**: 10x mais rápido que grep/read
- **Refatoração segura**: Garantia de não quebrar código
- **Navegação instantânea**: Jump to definition/implementation
- **Análise em tempo real**: Erros e warnings contextualizados

### Vantagem Competitiva
- Único MCP com rust-analyzer completo
- Experiência similar a IDEs
- Integração perfeita com Claude Code

## 📅 Cronograma

- **Semana 1**: Fases 1 e 2
- **Semana 2**: Fase 3
- **Semana 3**: Fase 4 e lançamento v2.0

## 🔄 Próximos Passos

1. Commit das correções atuais
2. Criar branch `feature/lsp-integration`
3. Implementar melhorias incrementalmente
4. Testar com projetos reais
5. Documentar novos recursos