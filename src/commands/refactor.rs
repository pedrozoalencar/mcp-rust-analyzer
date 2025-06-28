use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::debug;

use crate::analyzer::RustAnalyzer;
use crate::server::CommandHandler;

#[derive(Debug, Serialize, Deserialize)]
struct RenameParams {
    file: String,
    line: u32,
    column: u32,
    new_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExtractFunctionParams {
    file: String,
    start_line: u32,
    start_column: u32,
    end_line: u32,
    end_column: u32,
    function_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InlineParams {
    file: String,
    line: u32,
    column: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrganizeImportsParams {
    file: String,
}

pub struct RefactorCommands;

#[async_trait::async_trait]
impl CommandHandler for RefactorCommands {
    async fn handle(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let method = params
            .as_ref()
            .and_then(|p| p.get("method"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
            
        match method {
            "rename" => self.rename(params, analyzer).await,
            "extract_function" => self.extract_function(params, analyzer).await,
            "inline" => self.inline(params, analyzer).await,
            "organize_imports" => self.organize_imports(params, analyzer).await,
            _ => anyhow::bail!("Unknown refactor method: {}", method),
        }
    }
}

impl RefactorCommands {
    async fn rename(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params_value = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
        
        // Extract method field if present and remove it before parsing
        let mut params_value = params_value;
        if let Some(obj) = params_value.as_object_mut() {
            obj.remove("method");
        }
        
        let params: RenameParams = serde_json::from_value(params_value)?;
        
        debug!("Renaming at {}:{}:{} to {}", params.file, params.line, params.column, params.new_name);
        
        // Use the LSP-based rename functionality
        let changes = analyzer.rename(&params.file, params.line, params.column, &params.new_name).await?;
        
        Ok(json!({
            "file": params.file,
            "position": {
                "line": params.line,
                "column": params.column
            },
            "new_name": params.new_name,
            "changes": changes
        }))
    }
    
    async fn extract_function(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params_value = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
        
        // Extract method field if present and remove it before parsing
        let mut params_value = params_value;
        if let Some(obj) = params_value.as_object_mut() {
            obj.remove("method");
        }
        
        let params: ExtractFunctionParams = serde_json::from_value(params_value)?;
        
        debug!("Extracting function {} from {}:{}:{} to {}:{}", 
            params.function_name, params.file, 
            params.start_line, params.start_column,
            params.end_line, params.end_column
        );
        
        // Try to use LSP code actions for extract function
        if let Some(mut lsp_guard) = analyzer.get_lsp_client().await {
            if let Some(client) = lsp_guard.as_mut() {
            let full_path = if params.file.starts_with(&analyzer.project_root().to_string_lossy().to_string()) {
                params.file.clone()
            } else {
                analyzer.project_root().join(&params.file).to_string_lossy().to_string()
            };
            
            let _ = client.did_open(&full_path).await;
            
            let code_action_params = json!({
                "textDocument": {
                    "uri": format!("file://{}", full_path)
                },
                "range": {
                    "start": {
                        "line": params.start_line - 1,
                        "character": params.start_column - 1
                    },
                    "end": {
                        "line": params.end_line - 1,
                        "character": params.end_column - 1
                    }
                },
                "context": {
                    "diagnostics": [],
                    "only": ["refactor.extract"]
                }
            });
            
            match client.code_action(code_action_params).await {
                Ok(actions) => {
                    return Ok(json!({
                        "status": "lsp_code_actions_available",
                        "function_name": params.function_name,
                        "file": params.file,
                        "range": {
                            "start": {
                                "line": params.start_line,
                                "column": params.start_column
                            },
                            "end": {
                                "line": params.end_line,
                                "column": params.end_column
                            }
                        },
                        "available_actions": actions
                    }));
                }
                Err(e) => {
                    debug!("LSP code action failed: {}", e);
                }
            }
        }
        }
        
        // Fallback: Basic extract function implementation
        Ok(json!({
            "status": "basic_extract_function_placeholder",
            "function_name": params.function_name,
            "file": params.file,
            "range": {
                "start": {
                    "line": params.start_line,
                    "column": params.start_column
                },
                "end": {
                    "line": params.end_line,
                    "column": params.end_column
                }
            },
            "note": "Full LSP integration would provide automatic code generation"
        }))
    }
    
    async fn inline(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params_value = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
        
        // Extract method field if present and remove it before parsing
        let mut params_value = params_value;
        if let Some(obj) = params_value.as_object_mut() {
            obj.remove("method");
        }
        
        let params: InlineParams = serde_json::from_value(params_value)?;
        
        debug!("Inlining at {}:{}:{}", params.file, params.line, params.column);
        
        // Try to use LSP code actions for inline
        if let Some(mut lsp_guard) = analyzer.get_lsp_client().await {
            if let Some(client) = lsp_guard.as_mut() {
            let full_path = if params.file.starts_with(&analyzer.project_root().to_string_lossy().to_string()) {
                params.file.clone()
            } else {
                analyzer.project_root().join(&params.file).to_string_lossy().to_string()
            };
            
            let _ = client.did_open(&full_path).await;
            
            let code_action_params = json!({
                "textDocument": {
                    "uri": format!("file://{}", full_path)
                },
                "range": {
                    "start": {
                        "line": params.line - 1,
                        "character": params.column - 1
                    },
                    "end": {
                        "line": params.line - 1,
                        "character": params.column - 1
                    }
                },
                "context": {
                    "diagnostics": [],
                    "only": ["refactor.inline"]
                }
            });
            
            match client.code_action(code_action_params).await {
                Ok(actions) => {
                    return Ok(json!({
                        "status": "lsp_code_actions_available",
                        "file": params.file,
                        "position": {
                            "line": params.line,
                            "column": params.column
                        },
                        "available_actions": actions
                    }));
                }
                Err(e) => {
                    debug!("LSP code action failed: {}", e);
                }
            }
        }
        }
        
        Ok(json!({
            "status": "inline_placeholder",
            "file": params.file,
            "position": {
                "line": params.line,
                "column": params.column
            },
            "note": "Full LSP integration would provide automatic inlining"
        }))
    }
    
    async fn organize_imports(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params_value = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
        
        // Extract method field if present and remove it before parsing
        let mut params_value = params_value;
        if let Some(obj) = params_value.as_object_mut() {
            obj.remove("method");
        }
        
        let params: OrganizeImportsParams = serde_json::from_value(params_value)?;
        
        debug!("Organizing imports in {}", params.file);
        
        // Try to use LSP code actions for organize imports
        if let Some(mut lsp_guard) = analyzer.get_lsp_client().await {
            if let Some(client) = lsp_guard.as_mut() {
            let full_path = if params.file.starts_with(&analyzer.project_root().to_string_lossy().to_string()) {
                params.file.clone()
            } else {
                analyzer.project_root().join(&params.file).to_string_lossy().to_string()
            };
            
            let _ = client.did_open(&full_path).await;
            
            let code_action_params = json!({
                "textDocument": {
                    "uri": format!("file://{}", full_path)
                },
                "range": {
                    "start": {
                        "line": 0,
                        "character": 0
                    },
                    "end": {
                        "line": 0,
                        "character": 0
                    }
                },
                "context": {
                    "diagnostics": [],
                    "only": ["source.organizeImports"]
                }
            });
            
            match client.code_action(code_action_params).await {
                Ok(actions) => {
                    return Ok(json!({
                        "status": "lsp_code_actions_available",
                        "file": params.file,
                        "available_actions": actions
                    }));
                }
                Err(e) => {
                    debug!("LSP code action failed: {}", e);
                }
            }
        }
        }
        
        Ok(json!({
            "status": "organize_imports_placeholder",
            "file": params.file,
            "note": "Full LSP integration would provide automatic import organization"
        }))
    }
}