use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::debug;

use crate::analyzer::RustAnalyzer;
use crate::server::CommandHandler;

#[derive(Debug, Serialize, Deserialize)]
struct CompletionParams {
    file: String,
    line: u32,
    column: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContextParams {
    context: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImportParams {
    symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SnippetParams {
    name: String,
}

pub struct CompletionCommands;

#[async_trait::async_trait]
impl CommandHandler for CompletionCommands {
    async fn handle(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let method = params
            .as_ref()
            .and_then(|p| p.get("method"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
            
        match method {
            "complete" => self.complete(params, analyzer).await,
            "signature_help" => self.signature_help(params, analyzer).await,
            "get_completions" => self.get_completions(params, analyzer).await,
            "resolve_import" => self.resolve_import(params, analyzer).await,
            "expand_snippet" => self.expand_snippet(params, analyzer).await,
            _ => anyhow::bail!("Unknown completion method: {}", method),
        }
    }
}

impl CompletionCommands {
    async fn complete(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params: CompletionParams = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?
        )?;
        
        debug!("Getting completions at {}:{}:{}", params.file, params.line, params.column);
        
        // Use the new LSP-based completion functionality
        let completions = analyzer.completions(&params.file, params.line, params.column).await?;
        
        Ok(json!({
            "file": params.file,
            "position": {
                "line": params.line,
                "column": params.column
            },
            "completions": completions
        }))
    }
    
    async fn signature_help(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params: CompletionParams = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?
        )?;
        
        debug!("Getting signature help at {}:{}:{}", params.file, params.line, params.column);
        
        // TODO: Implement via LSP textDocument/signatureHelp
        // For now, return empty result
        Ok(json!({
            "file": params.file,
            "position": {
                "line": params.line,
                "column": params.column
            },
            "signatures": [],
            "activeSignature": null,
            "activeParameter": null
        }))
    }
    
    async fn get_completions(&self, params: Option<Value>, _analyzer: &RustAnalyzer) -> Result<Value> {
        let params: ContextParams = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?
        )?;
        
        // This would analyze the context string and provide relevant completions
        // For now, returning a placeholder
        Ok(json!({
            "context": params.context,
            "suggestions": [
                {
                    "label": "Vec::new()",
                    "kind": "method",
                    "detail": "Creates a new empty Vec"
                },
                {
                    "label": "HashMap::new()",
                    "kind": "method",
                    "detail": "Creates a new empty HashMap"
                }
            ]
        }))
    }
    
    async fn resolve_import(&self, params: Option<Value>, _analyzer: &RustAnalyzer) -> Result<Value> {
        let params: ImportParams = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?
        )?;
        
        debug!("Resolving import for symbol: {}", params.symbol);
        
        // Stub implementation for testing
        Ok(json!({
            "symbol": params.symbol,
            "imports": []
        }))
    }
    
    async fn expand_snippet(&self, params: Option<Value>, _analyzer: &RustAnalyzer) -> Result<Value> {
        let params: SnippetParams = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?
        )?;
        
        let snippet = match params.name.as_str() {
            "match_expr" => {
                r#"match ${1:expression} {
    ${2:pattern} => ${3:value},
    _ => ${4:default},
}"#
            }
            "if_let" => {
                r#"if let ${1:Some(value)} = ${2:expression} {
    ${3:// body}
}"#
            }
            "for_loop" => {
                r#"for ${1:item} in ${2:iterator} {
    ${3:// body}
}"#
            }
            "impl_trait" => {
                r#"impl ${1:Trait} for ${2:Type} {
    ${3:// implementation}
}"#
            }
            "test_fn" => {
                r#"#[test]
fn ${1:test_name}() {
    ${2:// test body}
}"#
            }
            _ => return Err(anyhow::anyhow!("Unknown snippet: {}", params.name))
        };
        
        Ok(json!({
            "name": params.name,
            "snippet": snippet
        }))
    }
}