use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::debug;
use std::path::Path;

use crate::analyzer::RustAnalyzer;
use crate::server::CommandHandler;

#[derive(Debug, Serialize, Deserialize)]
struct SymbolParams {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PositionParams {
    file: String,
    line: u32,
    column: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileParams {
    file: Option<String>,
}

pub struct AnalysisCommands;

#[async_trait::async_trait]
impl CommandHandler for AnalysisCommands {
    async fn handle(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let method = params
            .as_ref()
            .and_then(|p| p.get("method"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
            
        match method {
            "analyze_symbol" => self.analyze_symbol(params, analyzer).await,
            "find_references" => self.find_references(params, analyzer).await,
            "get_diagnostics" => self.get_diagnostics(params, analyzer).await,
            "get_hover" => self.get_hover(params, analyzer).await,
            "find_implementations" => self.find_implementations(params, analyzer).await,
            _ => anyhow::bail!("Unknown analysis method: {}", method),
        }
    }
}

impl AnalysisCommands {
    async fn analyze_symbol(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params_value = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
        
        // Extract method field if present and remove it before parsing
        let mut params_value = params_value;
        if let Some(obj) = params_value.as_object_mut() {
            obj.remove("method");
        }
        
        let params: SymbolParams = serde_json::from_value(params_value)?;
        
        debug!("Analyzing symbol: {}", params.name);
        
        // Use both LSP workspace symbols and local file search for comprehensive analysis
        let project_root = analyzer.project_root();
        let mut symbol_info = self.search_symbol_in_project(&params.name, project_root).await?;
        
        // Try to get additional info via LSP workspace symbols if available
        if let Some(mut lsp_guard) = analyzer.get_lsp_client().await {
            if let Some(client) = lsp_guard.as_mut() {
            if let Ok(lsp_symbols) = client.workspace_symbol(&params.name).await {
                if let Some(symbols) = lsp_symbols.as_array() {
                    for symbol in symbols {
                        if let Some(location) = symbol.get("location") {
                            if let Some(uri) = location.get("uri").and_then(|u| u.as_str()) {
                                // Convert file URI to relative path
                                let file_path = uri.strip_prefix("file://")
                                    .unwrap_or(uri)
                                    .strip_prefix(&project_root.to_string_lossy().to_string())
                                    .unwrap_or(uri);
                                    
                                let range = location.get("range");
                                let line = range.and_then(|r| r.get("start"))
                                    .and_then(|s| s.get("line"))
                                    .and_then(|l| l.as_u64())
                                    .unwrap_or(0) + 1; // Convert from 0-based to 1-based
                                
                                symbol_info.push(json!({
                                    "file": file_path,
                                    "line": line,
                                    "content": symbol.get("name").unwrap_or(&json!("")),
                                    "context": "lsp_workspace_symbol",
                                    "kind": symbol.get("kind").unwrap_or(&json!("unknown")),
                                    "container": symbol.get("containerName").unwrap_or(&json!(""))
                                }));
                            }
                        }
                    }
                }
            }
        }
        }
        
        // Deduplicate results
        let mut unique_locations = Vec::new();
        let mut seen = std::collections::HashSet::new();
        
        for location in symbol_info {
            let key = format!("{}:{}", 
                location.get("file").unwrap_or(&json!("")).as_str().unwrap_or(""),
                location.get("line").unwrap_or(&json!(0)).as_u64().unwrap_or(0)
            );
            
            if seen.insert(key) {
                unique_locations.push(location);
            }
        }
        
        // Analyze symbol type based on patterns
        let symbol_type = if params.name.chars().next().unwrap_or('a').is_uppercase() {
            if params.name.contains("Error") || params.name.contains("Exception") {
                "Error Type"
            } else if unique_locations.iter().any(|loc| 
                loc.get("context").and_then(|c| c.as_str()).unwrap_or("").contains("struct")
            ) {
                "Struct"
            } else if unique_locations.iter().any(|loc| 
                loc.get("context").and_then(|c| c.as_str()).unwrap_or("").contains("enum")
            ) {
                "Enum"
            } else if unique_locations.iter().any(|loc| 
                loc.get("context").and_then(|c| c.as_str()).unwrap_or("").contains("trait")
            ) {
                "Trait"
            } else {
                "Type"
            }
        } else if params.name.starts_with("is_") || params.name.starts_with("has_") || params.name.starts_with("can_") {
            "Predicate Function"
        } else if params.name.ends_with("_mut") {
            "Mutable Function/Method"
        } else {
            "Function/Variable"
        };
        
        let lsp_status = {
            let lsp_client_guard = analyzer.get_lsp_client().await;
            let has_lsp = lsp_client_guard.as_ref().map(|g| g.as_ref().is_some()).unwrap_or(false);
            if has_lsp { "lsp_workspace_symbols" } else { "lsp_unavailable" }
        };
        
        Ok(json!({
            "symbol": params.name,
            "occurrences": unique_locations.len(),
            "locations": unique_locations,
            "analysis": {
                "type": symbol_type,
                "status": "enhanced_analysis_complete",
                "search_methods": [
                    "file_content_search",
                    lsp_status
                ]
            }
        }))
    }
    
    async fn find_references(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params: PositionParams = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?
        )?;
        
        // Use the new LSP-based references functionality
        let references = analyzer.find_references(&params.file, params.line, params.column).await?;
        
        Ok(json!({
            "file": params.file,
            "position": {
                "line": params.line,
                "column": params.column
            },
            "references": references
        }))
    }
    
    async fn get_diagnostics(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let mut params_value = params.unwrap_or(json!({}));
        
        // Extract method field if present and remove it before parsing
        if let Some(obj) = params_value.as_object_mut() {
            obj.remove("method");
        }
        
        let params: FileParams = serde_json::from_value(params_value)?;
        
        debug!("Getting diagnostics for file: {:?}", params.file);
        
        let mut diagnostics = Vec::new();
        let mut sources: Vec<String> = Vec::new();
        
        // If specific file is requested, try to get LSP diagnostics
        if let Some(file) = &params.file {
            if let Some(mut lsp_guard) = analyzer.get_lsp_client().await {
                if let Some(client) = lsp_guard.as_mut() {
                let full_path = if file.starts_with(&analyzer.project_root().to_string_lossy().to_string()) {
                    file.clone()
                } else {
                    analyzer.project_root().join(file).to_string_lossy().to_string()
                };
                
                // Ensure document is open to get fresh diagnostics
                let _ = client.did_open(&full_path).await;
                
                    // Note: LSP diagnostics are usually pushed via notifications
                    // For now, we'll use cargo check as fallback
                    sources.push("lsp_integration_pending".to_string());
                }
            }
        }
        
        // Always try cargo check for comprehensive diagnostics
        use tokio::process::Command;
        
        let cargo_output = Command::new("cargo")
            .args(&["check", "--message-format=json"])
            .current_dir(analyzer.project_root())
            .output()
            .await;
            
        match cargo_output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                sources.push("cargo_check".to_string());
                
                // Parse cargo output for diagnostics
                for line in stdout.lines() {
                    if let Ok(json_msg) = serde_json::from_str::<Value>(line) {
                        if json_msg.get("reason") == Some(&json!("compiler-message")) {
                            if let Some(message) = json_msg.get("message") {
                                if let Some(spans) = message.get("spans").and_then(|s| s.as_array()) {
                                    for span in spans {
                                        if let Some(file_name) = span.get("file_name").and_then(|f| f.as_str()) {
                                            // Filter by file if specified
                                            if params.file.is_none() || 
                                               params.file.as_ref().map_or(false, |f| file_name.contains(f) || f.contains(file_name)) {
                                                diagnostics.push(json!({
                                                    "file": file_name,
                                                    "line": span.get("line_start"),
                                                    "column": span.get("column_start"),
                                                    "level": message.get("level").unwrap_or(&json!("error")),
                                                    "message": message.get("message").unwrap_or(&json!("")),
                                                    "code": message.get("code"),
                                                    "source": "cargo"
                                                }));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                sources.push(format!("cargo_check_failed: {}", e));
            }
        }
        
        Ok(json!({ 
            "file": params.file,
            "diagnostics": diagnostics,
            "total_diagnostics": diagnostics.len(),
            "sources": sources,
            "status": "cargo_check_complete",
            "note": "LSP real-time diagnostics require notification handling"
        }))
    }
    
    async fn get_hover(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params: PositionParams = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?
        )?;
        
        // Use the new LSP-based hover functionality
        let hover_text = analyzer.hover(&params.file, params.line, params.column).await?;
        
        Ok(json!({ 
            "contents": hover_text,
            "file": params.file,
            "line": params.line,
            "column": params.column
        }))
    }
    
    async fn find_implementations(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params: PositionParams = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?
        )?;
        
        debug!("Finding implementations at {}:{}:{}", params.file, params.line, params.column);
        
        // Use the new LSP-based implementations functionality
        let implementations = analyzer.find_implementations(&params.file, params.line, params.column).await?;
        
        Ok(json!({
            "file": params.file,
            "position": {
                "line": params.line,
                "column": params.column
            },
            "implementations": implementations
        }))
    }
    
    async fn search_symbol_in_project(&self, symbol: &str, project_root: &std::path::Path) -> Result<Vec<Value>> {
        let mut locations = Vec::new();
        
        // Search in src directory
        let src_path = project_root.join("src");
        if src_path.is_dir() {
            self.search_symbol_in_directory(symbol, &src_path, &mut locations).await?;
        }
        
        Ok(locations)
    }
    
    async fn search_symbol_in_directory(&self, symbol: &str, dir: &Path, locations: &mut Vec<Value>) -> Result<()> {
        use std::fs;
        
        let entries = fs::read_dir(dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively search subdirectories
                Box::pin(self.search_symbol_in_directory(symbol, &path, locations)).await?;
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                // Search in Rust files
                if let Ok(content) = fs::read_to_string(&path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.contains(symbol) {
                            locations.push(json!({
                                "file": path.strip_prefix(path.parent().unwrap().parent().unwrap()).unwrap_or(&path).display().to_string(),
                                "line": line_num + 1,
                                "content": line.trim(),
                                "context": "code"
                            }));
                            
                            // Limit results to avoid too much data
                            if locations.len() >= 50 {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}