use anyhow::{Result, Context};
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
            
            // Handle MCP protocol methods
            let response = match method {
                "initialize" => self.handle_initialize(id, params).await,
                "tools/list" => self.handle_tools_list(id).await,
                "tools/call" => self.handle_tools_call(id, params).await,
                "resources/list" => self.handle_resources_list(id).await,
                "resources/read" => self.handle_resources_read(id, params).await,
                "prompts/list" => self.handle_prompts_list(id).await,
                "prompts/get" => self.handle_prompts_get(id, params).await,
                "completion/complete" => self.handle_completion_complete(id, params).await,
                _ => {
                    // Handle custom methods
                    if let Some(handler) = self.commands.get(method) {
                debug!("Found handler for method: {}", method);
                match handler.handle(params, &self.analyzer).await {
                    Ok(result) => {
                        debug!("Handler returned result: {:?}", result);
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": result
                        })
                    },
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
            }
                }
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
    
    async fn handle_initialize(&self, id: &Value, _params: Option<Value>) -> Value {
        info!("Handling initialize request");
        // Start LSP initialization in background after responding
        self.analyzer.start_lsp_initialization().await;
        info!("LSP initialization triggered");
        
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {},
                    "completion": {}
                },
                "serverInfo": {
                    "name": "mcp-rust-analyzer",
                    "version": "0.1.0"
                }
            }
        })
    }
    
    async fn handle_tools_list(&self, id: &Value) -> Value {
        let mut tools = Vec::new();
        
        // Add all our commands as tools
        tools.push(json!({
            "name": "project_structure",
            "description": "Analyze the project structure and module organization",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }));
        
        tools.push(json!({
            "name": "code_metrics", 
            "description": "Get code metrics for a module or the entire project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module": {
                        "type": "string",
                        "description": "Module path to analyze (e.g., 'src' or '.')"
                    }
                },
                "required": []
            }
        }));
        
        tools.push(json!({
            "name": "analyze_dependencies",
            "description": "Analyze project dependencies from Cargo.toml",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }));
        
        tools.push(json!({
            "name": "expand_snippet",
            "description": "Expand a code snippet template",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Snippet name (match_expr, if_let, for_loop, impl_trait, test_fn)"
                    }
                },
                "required": ["name"]
            }
        }));
        
        tools.push(json!({
            "name": "get_hover",
            "description": "Get hover information for a position in a file",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "File path relative to project root"
                    },
                    "line": {
                        "type": "number",
                        "description": "Line number (1-based)"
                    },
                    "column": {
                        "type": "number",
                        "description": "Column number (1-based)"
                    }
                },
                "required": ["file", "line", "column"]
            }
        }));
        
        tools.push(json!({
            "name": "complete",
            "description": "Get code completions at a position",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "File path relative to project root"
                    },
                    "line": {
                        "type": "number",
                        "description": "Line number (1-based)"
                    },
                    "column": {
                        "type": "number",
                        "description": "Column number (1-based)"
                    }
                },
                "required": ["file", "line", "column"]
            }
        }));
        
        tools.push(json!({
            "name": "find_references",
            "description": "Find all references to a symbol at a position",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "File path relative to project root"
                    },
                    "line": {
                        "type": "number",
                        "description": "Line number (1-based)"
                    },
                    "column": {
                        "type": "number",
                        "description": "Column number (1-based)"
                    }
                },
                "required": ["file", "line", "column"]
            }
        }));
        
        tools.push(json!({
            "name": "rename",
            "description": "Rename a symbol at a position",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "File path relative to project root"
                    },
                    "line": {
                        "type": "number",
                        "description": "Line number (1-based)"
                    },
                    "column": {
                        "type": "number",
                        "description": "Column number (1-based)"
                    },
                    "new_name": {
                        "type": "string",
                        "description": "New name for the symbol"
                    }
                },
                "required": ["file", "line", "column", "new_name"]
            }
        }));
        
        // Additional IntelliSense tools
        tools.push(json!({
            "name": "signature_help",
            "description": "Get signature help for function calls",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "File path relative to project root"
                    },
                    "line": {
                        "type": "number",
                        "description": "Line number (1-based)"
                    },
                    "column": {
                        "type": "number",
                        "description": "Column number (1-based)"
                    }
                },
                "required": ["file", "line", "column"]
            }
        }));
        
        tools.push(json!({
            "name": "get_diagnostics",
            "description": "Get diagnostics for a file or the entire project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "File path (optional, omit for project-wide)"
                    }
                },
                "required": []
            }
        }));
        
        tools.push(json!({
            "name": "analyze_symbol",
            "description": "Analyze a symbol by name across the project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Symbol name to analyze"
                    }
                },
                "required": ["name"]
            }
        }));
        
        tools.push(json!({
            "name": "find_implementations",
            "description": "Find implementations of a trait or type",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "File path relative to project root"
                    },
                    "line": {
                        "type": "number",
                        "description": "Line number (1-based)"
                    },
                    "column": {
                        "type": "number",
                        "description": "Column number (1-based)"
                    }
                },
                "required": ["file", "line", "column"]
            }
        }));
        
        tools.push(json!({
            "name": "find_dead_code",
            "description": "Find unused code in the project",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }));
        
        tools.push(json!({
            "name": "suggest_improvements",
            "description": "Get improvement suggestions for a module",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module": {
                        "type": "string",
                        "description": "Module path to analyze"
                    }
                },
                "required": []
            }
        }));
        
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools
            }
        })
    }
    
    async fn handle_tools_call(&self, id: &Value, params: Option<Value>) -> Value {
        if let Some(params) = params {
            if let Some(name) = params.get("name").and_then(|v| v.as_str()) {
                let args = params.get("arguments").cloned();
                
                // Map tool name to method name and add method field
                let mut method_params = args.unwrap_or(json!({}));
                if let Some(obj) = method_params.as_object_mut() {
                    obj.insert("method".to_string(), json!(name));
                }
                
                // Call the appropriate handler
                if let Some(handler) = self.commands.get(name) {
                    match handler.handle(Some(method_params), &self.analyzer).await {
                        Ok(result) => {
                            return json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [{
                                        "type": "text",
                                        "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                                    }]
                                }
                            });
                        }
                        Err(e) => {
                            return json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {
                                    "code": -32603,
                                    "message": format!("Tool execution failed: {}", e)
                                }
                            });
                        }
                    }
                }
            }
        }
        
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32602,
                "message": "Invalid params"
            }
        })
    }
    
    async fn handle_resources_list(&self, id: &Value) -> Value {
        let mut resources = Vec::new();
        
        // Project-wide resources
        resources.push(json!({
            "uri": "rust-analyzer://project/structure",
            "name": "Project Structure",
            "description": "Current project structure and modules",
            "mimeType": "application/json"
        }));
        
        resources.push(json!({
            "uri": "rust-analyzer://project/diagnostics",
            "name": "Project Diagnostics",
            "description": "All diagnostics for the project",
            "mimeType": "application/json"
        }));
        
        resources.push(json!({
            "uri": "rust-analyzer://project/dependencies",
            "name": "Dependencies Graph",
            "description": "Project dependencies and their relationships",
            "mimeType": "application/json"
        }));
        
        resources.push(json!({
            "uri": "rust-analyzer://project/symbols",
            "name": "Workspace Symbols",
            "description": "All symbols in the workspace",
            "mimeType": "application/json"
        }));
        
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "resources": resources
            }
        })
    }
    
    async fn handle_resources_read(&self, id: &Value, params: Option<Value>) -> Value {
        if let Some(params) = params {
            if let Some(uri) = params.get("uri").and_then(|v| v.as_str()) {
                let result = match uri {
                    "rust-analyzer://project/structure" => {
                        // Call our existing project_structure command
                        if let Some(handler) = self.commands.get("project_structure") {
                            match handler.handle(Some(json!({"method": "project_structure"})), &self.analyzer).await {
                                Ok(data) => json!({
                                    "contents": [{
                                        "uri": uri,
                                        "mimeType": "application/json",
                                        "text": serde_json::to_string_pretty(&data).unwrap_or_default()
                                    }]
                                }),
                                Err(e) => return json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "error": {
                                        "code": -32603,
                                        "message": format!("Failed to read resource: {}", e)
                                    }
                                })
                            }
                        } else {
                            return json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {
                                    "code": -32603,
                                    "message": "Resource handler not found"
                                }
                            })
                        }
                    },
                    "rust-analyzer://project/dependencies" => {
                        if let Some(handler) = self.commands.get("analyze_dependencies") {
                            match handler.handle(Some(json!({"method": "analyze_dependencies"})), &self.analyzer).await {
                                Ok(data) => json!({
                                    "contents": [{
                                        "uri": uri,
                                        "mimeType": "application/json",
                                        "text": serde_json::to_string_pretty(&data).unwrap_or_default()
                                    }]
                                }),
                                Err(e) => return json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "error": {
                                        "code": -32603,
                                        "message": format!("Failed to read resource: {}", e)
                                    }
                                })
                            }
                        } else {
                            return json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {
                                    "code": -32603,
                                    "message": "Resource handler not found"
                                }
                            })
                        }
                    },
                    _ => {
                        return json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32602,
                                "message": format!("Unknown resource URI: {}", uri)
                            }
                        })
                    }
                };
                
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result
                });
            }
        }
        
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32602,
                "message": "Invalid params: missing uri"
            }
        })
    }
    
    async fn handle_prompts_list(&self, id: &Value) -> Value {
        let mut prompts = Vec::new();
        
        // IntelliSense prompts
        prompts.push(json!({
            "name": "analyze_code",
            "description": "Analyze Rust code and provide IntelliSense information",
            "arguments": [{
                "name": "file",
                "description": "File path to analyze",
                "required": true
            }, {
                "name": "position",
                "description": "Cursor position (line:column)",
                "required": false
            }]
        }));
        
        prompts.push(json!({
            "name": "refactor_code",
            "description": "Suggest and apply refactorings",
            "arguments": [{
                "name": "operation",
                "description": "Type of refactoring (extract_function, rename, inline)",
                "required": true
            }, {
                "name": "context",
                "description": "Code context for refactoring",
                "required": true
            }]
        }));
        
        prompts.push(json!({
            "name": "explain_error",
            "description": "Explain a Rust compiler error with suggestions",
            "arguments": [{
                "name": "error",
                "description": "The error message to explain",
                "required": true
            }]
        }));
        
        prompts.push(json!({
            "name": "optimize_code",
            "description": "Suggest performance optimizations",
            "arguments": [{
                "name": "code",
                "description": "Code to optimize",
                "required": true
            }, {
                "name": "metrics",
                "description": "Include performance metrics",
                "required": false
            }]
        }));
        
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "prompts": prompts
            }
        })
    }
    
    async fn handle_prompts_get(&self, id: &Value, params: Option<Value>) -> Value {
        if let Some(params) = params {
            if let Some(name) = params.get("name").and_then(|v| v.as_str()) {
                let prompt = match name {
                    "analyze_code" => {
                        let file = params.get("arguments")
                            .and_then(|args| args.get("file"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        
                        json!({
                            "description": "Analyze Rust code with IntelliSense",
                            "messages": [{
                                "role": "user",
                                "content": {
                                    "type": "text",
                                    "text": format!("Please analyze the Rust code in {} and provide:\n\
                                        1. Type information for symbols\n\
                                        2. Available methods and completions\n\
                                        3. Documentation for APIs\n\
                                        4. Any potential issues or improvements\n\
                                        \n\
                                        Use the rust-analyzer tools to gather this information.", file)
                                }
                            }]
                        })
                    },
                    "refactor_code" => {
                        let operation = params.get("arguments")
                            .and_then(|args| args.get("operation"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("rename");
                        
                        json!({
                            "description": "Refactor Rust code",
                            "messages": [{
                                "role": "user",
                                "content": {
                                    "type": "text",
                                    "text": format!("Please perform a {} refactoring on the selected code. \
                                        Use rust-analyzer to ensure the refactoring is safe and preserves behavior.", operation)
                                }
                            }]
                        })
                    },
                    "explain_error" => {
                        json!({
                            "description": "Explain Rust compiler error",
                            "messages": [{
                                "role": "user",
                                "content": {
                                    "type": "text",
                                    "text": "Please explain this Rust compiler error and suggest fixes. \
                                        Use rust-analyzer diagnostics to provide accurate information."
                                }
                            }]
                        })
                    },
                    "optimize_code" => {
                        json!({
                            "description": "Optimize Rust code",
                            "messages": [{
                                "role": "user",
                                "content": {
                                    "type": "text",
                                    "text": "Please analyze this Rust code for performance optimizations. \
                                        Consider: memory usage, algorithmic complexity, idiomatic Rust patterns, \
                                        and potential parallelization opportunities."
                                }
                            }]
                        })
                    },
                    _ => {
                        return json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32602,
                                "message": format!("Unknown prompt: {}", name)
                            }
                        })
                    }
                };
                
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": prompt
                });
            }
        }
        
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32602,
                "message": "Invalid params: missing name"
            }
        })
    }
    
    async fn handle_completion_complete(&self, id: &Value, params: Option<Value>) -> Value {
        // This is for auto-completion support
        if let Some(params) = params {
            if let Some(ref_value) = params.get("ref") {
                // Handle completion based on the reference
                let completion_text = match ref_value.get("type").and_then(|v| v.as_str()) {
                    Some("ref/resource") => {
                        if let Some(uri) = ref_value.get("uri").and_then(|v| v.as_str()) {
                            format!("Resource: {}", uri)
                        } else {
                            "Resource reference".to_string()
                        }
                    },
                    Some("ref/prompt") => {
                        if let Some(name) = ref_value.get("name").and_then(|v| v.as_str()) {
                            format!("Prompt: {}", name)
                        } else {
                            "Prompt reference".to_string()
                        }
                    },
                    _ => "Unknown reference type".to_string()
                };
                
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "completion": {
                            "values": [completion_text],
                            "total": 1,
                            "hasMore": false
                        }
                    }
                });
            }
        }
        
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32602,
                "message": "Invalid params for completion"
            }
        })
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