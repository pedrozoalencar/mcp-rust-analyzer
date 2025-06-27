use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;
use tokio::sync::Mutex;
use serde_json::{json, Value};

use crate::lsp_client::{LspClient, LspClientConfig};

// Temporary stub types for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u32);

#[derive(Debug, Clone)]
pub struct FilePosition {
    pub file_id: FileId,
    pub offset: TextSize,
}

#[derive(Debug, Clone)]
pub struct FileRange {
    pub file_id: FileId,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy)]
pub struct TextSize(u32);

#[derive(Debug, Clone)]
pub struct TextRange {
    start: TextSize,
    end: TextSize,
}

impl TextRange {
    pub fn new(start: TextSize, end: TextSize) -> Self {
        Self { start, end }
    }
}

// Stub types that will be replaced by real rust-analyzer types
pub struct Analysis;
pub struct AnalysisHost;
pub struct Vfs;
pub struct VfsPath;

pub struct RustAnalyzer {
    project_root: PathBuf,
    lsp_client: Arc<Mutex<Option<LspClient>>>,
    use_lsp: bool,
    // Legacy fields for compatibility
    host: AnalysisHost,
    analysis: Analysis,
    vfs: Arc<Vfs>,
    file_counter: u32,
}

impl RustAnalyzer {
    pub async fn new(project_path: &str) -> Result<Self> {
        info!("Initializing Rust Analyzer for project: {}", project_path);
        
        let project_root = PathBuf::from(project_path);
        let manifest_path = project_root.join("Cargo.toml");
        
        if !manifest_path.exists() {
            anyhow::bail!("No Cargo.toml found in project root");
        }
        
        // Check if we should use LSP client
        let use_lsp = std::env::var("USE_LSP").unwrap_or_default() == "true";
        
        let lsp_client = if use_lsp {
            info!("Using LSP backend");
            let config = LspClientConfig {
                server_path: "rust-analyzer".to_string(),
                server_args: vec![],
                root_path: project_root.clone(),
            };
            
            match LspClient::new(config) {
                Ok(mut client) => {
                    // Try to initialize
                    match client.initialize().await {
                        Ok(_) => Some(client),
                        Err(e) => {
                            info!("Failed to initialize LSP client: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    info!("Failed to create LSP client: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Temporary stub implementation
        let host = AnalysisHost;
        let vfs = Vfs;
        let analysis = Analysis;
        
        Ok(Self {
            project_root,
            lsp_client: Arc::new(Mutex::new(lsp_client)),
            use_lsp,
            host,
            analysis,
            vfs: Arc::new(vfs),
            file_counter: 0,
        })
    }
    
    pub async fn hover(&self, file_path: &str, line: u32, column: u32) -> Result<Option<String>> {
        if let Some(client) = self.lsp_client.lock().await.as_mut() {
            let params = json!({
                "textDocument": {
                    "uri": format!("file://{}/{}", self.project_root.display(), file_path)
                },
                "position": {
                    "line": line - 1,  // LSP uses 0-based
                    "character": column - 1
                }
            });
            
            match client.hover(params).await {
                Ok(result) => {
                    if let Some(contents) = result.get("contents") {
                        if let Some(value) = contents.get("value").and_then(|v| v.as_str()) {
                            return Ok(Some(value.to_string()));
                        } else if let Some(value) = contents.as_str() {
                            return Ok(Some(value.to_string()));
                        }
                    }
                    Ok(None)
                }
                Err(e) => {
                    info!("LSP hover failed: {}", e);
                    Ok(None)
                }
            }
        } else {
            // Fallback to stub implementation
            Ok(None)
        }
    }
    
    pub async fn completions(&self, file_path: &str, line: u32, column: u32) -> Result<Vec<Value>> {
        if let Some(client) = self.lsp_client.lock().await.as_mut() {
            let params = json!({
                "textDocument": {
                    "uri": format!("file://{}/{}", self.project_root.display(), file_path)
                },
                "position": {
                    "line": line - 1,  // LSP uses 0-based
                    "character": column - 1
                }
            });
            
            match client.completion(params).await {
                Ok(result) => {
                    if let Some(items) = result.get("items").and_then(|v| v.as_array()) {
                        return Ok(items.clone());
                    } else if result.is_array() {
                        return Ok(result.as_array().unwrap().clone());
                    }
                    Ok(Vec::new())
                }
                Err(e) => {
                    info!("LSP completion failed: {}", e);
                    Ok(Vec::new())
                }
            }
        } else {
            // Fallback to stub implementation
            Ok(Vec::new())
        }
    }
    
    pub async fn find_references(&self, file_path: &str, line: u32, column: u32) -> Result<Vec<Value>> {
        if let Some(client) = self.lsp_client.lock().await.as_mut() {
            let params = json!({
                "textDocument": {
                    "uri": format!("file://{}/{}", self.project_root.display(), file_path)
                },
                "position": {
                    "line": line - 1,  // LSP uses 0-based
                    "character": column - 1
                },
                "context": {
                    "includeDeclaration": true
                }
            });
            
            match client.references(params).await {
                Ok(result) => {
                    if let Some(refs) = result.as_array() {
                        return Ok(refs.clone());
                    }
                    Ok(Vec::new())
                }
                Err(e) => {
                    info!("LSP references failed: {}", e);
                    Ok(Vec::new())
                }
            }
        } else {
            // Fallback to stub implementation
            Ok(Vec::new())
        }
    }
    
    pub async fn rename(&self, file_path: &str, line: u32, column: u32, new_name: &str) -> Result<Value> {
        if let Some(client) = self.lsp_client.lock().await.as_mut() {
            let params = json!({
                "textDocument": {
                    "uri": format!("file://{}/{}", self.project_root.display(), file_path)
                },
                "position": {
                    "line": line - 1,  // LSP uses 0-based
                    "character": column - 1
                },
                "newName": new_name
            });
            
            match client.rename(params).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    info!("LSP rename failed: {}", e);
                    Ok(json!({}))
                }
            }
        } else {
            // Fallback to stub implementation
            Ok(json!({}))
        }
    }
    
    pub async fn rename_symbol(&self, _old_name: &str, _new_name: &str) -> Result<Value> {
        // This is a simplified implementation. In a real scenario, we'd need to:
        // 1. Search for the symbol across all files
        // 2. Determine its location
        // 3. Use that location to initiate the rename
        
        // For now, return a placeholder
        Ok(json!({
            "status": "not_implemented",
            "message": "Symbol-based rename requires full project search implementation"
        }))
    }
    
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
    
    // Legacy methods for compatibility
    pub fn get_file_id(&self, _file_path: &str) -> Result<FileId> {
        // Temporary stub implementation
        Ok(FileId(0))
    }
    
    pub fn get_file_position(&self, file_path: &str, line: u32, column: u32) -> Result<FilePosition> {
        let file_id = self.get_file_id(file_path)?;
        let offset = self.line_col_to_offset(file_id, line, column)?;
        
        Ok(FilePosition { file_id, offset })
    }
    
    pub fn get_file_range(&self, file_path: &str, start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Result<FileRange> {
        let file_id = self.get_file_id(file_path)?;
        let start = self.line_col_to_offset(file_id, start_line, start_col)?;
        let end = self.line_col_to_offset(file_id, end_line, end_col)?;
        
        Ok(FileRange { 
            file_id, 
            range: TextRange::new(start, end) 
        })
    }
    
    fn line_col_to_offset(&self, _file_id: FileId, _line: u32, _column: u32) -> Result<TextSize> {
        // Temporary stub implementation
        Ok(TextSize(0))
    }
    
    pub fn analysis(&self) -> &Analysis {
        &self.analysis
    }
    
    pub fn reload_workspace(&mut self) -> Result<()> {
        info!("Reloading workspace");
        // Implementation for reloading workspace after file changes
        Ok(())
    }
    
    pub fn get_all_files(&self) -> Vec<(FileId, PathBuf)> {
        // Temporary stub implementation
        vec![(FileId(0), self.project_root.join("src/lib.rs"))]
    }
}

impl Drop for RustAnalyzer {
    fn drop(&mut self) {
        // Ensure LSP client is properly shutdown
        if let Ok(mut guard) = self.lsp_client.try_lock() {
            if let Some(mut client) = guard.take() {
                let _ = tokio::runtime::Handle::try_current()
                    .map(|_| {
                        tokio::spawn(async move {
                            let _ = client.shutdown().await;
                        });
                    });
            }
        }
    }
}