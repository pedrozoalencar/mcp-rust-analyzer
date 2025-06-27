use anyhow::{Result, Context};
use jsonrpc_lite::JsonRpc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, debug};

use crate::analyzer::RustAnalyzer;
use crate::commands::{
    analysis::AnalysisCommands,
    completion::CompletionCommands,
    refactor::RefactorCommands,
    metrics::MetricsCommands,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub result: Option<Value>,
    pub error: Option<String>,
}

// Internal struct to deserialize Request
#[derive(Debug, Deserialize)]
struct RequestData {
    id: jsonrpc_lite::Id,
    method: String,
    params: Option<jsonrpc_lite::Params>,
}

// Internal struct to deserialize Notification
#[derive(Debug, Deserialize)]
struct NotificationData {
    method: String,
    params: Option<jsonrpc_lite::Params>,
}

pub struct McpServer {
    analyzer: RustAnalyzer,
    commands: HashMap<String, Box<dyn CommandHandler>>,
}

#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    async fn handle(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value>;
}

impl McpServer {
    pub async fn new(project_path: &str) -> Result<Self> {
        info!("Initializing MCP server for project: {}", project_path);
        
        let analyzer = RustAnalyzer::new(project_path).await?;
        let mut commands: HashMap<String, Box<dyn CommandHandler>> = HashMap::new();
        
        // Register analysis commands
        commands.insert("analyze_symbol".to_string(), Box::new(AnalysisCommands));
        commands.insert("find_references".to_string(), Box::new(AnalysisCommands));
        commands.insert("get_diagnostics".to_string(), Box::new(AnalysisCommands));
        commands.insert("get_hover".to_string(), Box::new(AnalysisCommands));
        commands.insert("find_implementations".to_string(), Box::new(AnalysisCommands));
        
        // Register completion commands
        commands.insert("complete".to_string(), Box::new(CompletionCommands));
        commands.insert("signature_help".to_string(), Box::new(CompletionCommands));
        commands.insert("get_completions".to_string(), Box::new(CompletionCommands));
        commands.insert("resolve_import".to_string(), Box::new(CompletionCommands));
        commands.insert("expand_snippet".to_string(), Box::new(CompletionCommands));
        
        // Register refactoring commands
        commands.insert("rename".to_string(), Box::new(RefactorCommands));
        commands.insert("extract_function".to_string(), Box::new(RefactorCommands));
        commands.insert("inline".to_string(), Box::new(RefactorCommands));
        commands.insert("organize_imports".to_string(), Box::new(RefactorCommands));
        
        // Register metrics commands
        commands.insert("project_structure".to_string(), Box::new(MetricsCommands));
        commands.insert("analyze_dependencies".to_string(), Box::new(MetricsCommands));
        commands.insert("code_metrics".to_string(), Box::new(MetricsCommands));
        commands.insert("find_dead_code".to_string(), Box::new(MetricsCommands));
        commands.insert("suggest_improvements".to_string(), Box::new(MetricsCommands));
        
        Ok(Self { analyzer, commands })
    }
    
    pub async fn handle_request(&self, request_str: &str) -> Result<String> {
        debug!("Received request: {}", request_str);
        
        // First parse as generic JSON to extract method and params
        let json_value: Value = serde_json::from_str(request_str)?;
        
        // Check if it's a request or notification
        if let Some(id) = json_value.get("id") {
            // It's a request
            let method = json_value.get("method")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing method"))?;
            let params = json_value.get("params").cloned();
            
            let response = if let Some(handler) = self.commands.get(method) {
                match handler.handle(params, &self.analyzer).await {
                    Ok(result) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result
                    }),
                    Err(e) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32603,
                            "message": format!("Command failed: {}", e)
                        }
                    })
                }
            } else {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                })
            };
            
            serde_json::to_string(&response).context("Failed to serialize response")
        } else {
            // It's a notification
            if let Some(method) = json_value.get("method").and_then(|v| v.as_str()) {
                info!("Received notification: {}", method);
            }
            Ok("".to_string())
        }
    }
    
    pub async fn capabilities(&self) -> Value {
        json!({
            "name": "mcp-rust-analyzer",
            "version": "0.1.0",
            "capabilities": {
                "analysis": [
                    "analyze_symbol",
                    "find_references",
                    "get_diagnostics",
                    "get_hover",
                    "find_implementations"
                ],
                "completion": [
                    "complete",
                    "signature_help",
                    "get_completions",
                    "resolve_import",
                    "expand_snippet"
                ],
                "refactoring": [
                    "rename",
                    "extract_function",
                    "inline",
                    "organize_imports"
                ],
                "metrics": [
                    "project_structure",
                    "analyze_dependencies",
                    "code_metrics",
                    "find_dead_code",
                    "suggest_improvements"
                ]
            }
        })
    }
}