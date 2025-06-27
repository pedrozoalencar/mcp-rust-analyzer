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
        let params: SymbolParams = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?
        )?;
        
        debug!("Analyzing symbol: {}", params.name);
        
        // For now, we'll provide a basic analysis based on project structure
        // In the future, this could be enhanced with LSP workspace symbol search
        let project_root = analyzer.project_root();
        let symbol_info = self.search_symbol_in_project(&params.name, project_root).await?;
        
        Ok(json!({
            "symbol": params.name,
            "occurrences": symbol_info.len(),
            "locations": symbol_info,
            "analysis": {
                "type": if params.name.chars().next().unwrap_or('a').is_uppercase() {
                    "Type/Struct/Enum"
                } else {
                    "function/variable"
                },
                "status": "basic_analysis_complete"
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
    
    async fn get_diagnostics(&self, params: Option<Value>, _analyzer: &RustAnalyzer) -> Result<Value> {
        let params: FileParams = params
            .map(|p| serde_json::from_value(p))
            .transpose()?
            .unwrap_or(FileParams { file: None });
        
        debug!("Getting diagnostics for file: {:?}", params.file);
        
        // TODO: Implement via LSP textDocument/publishDiagnostics
        Ok(json!({ 
            "file": params.file,
            "diagnostics": [],
            "status": "diagnostics_pending",
            "message": "Diagnostics integration requires LSP notification handling"
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