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
        
        // Check if we should use LSP client (default: true)
        let use_lsp = std::env::var("USE_LSP")
            .map(|v| v == "true")
            .unwrap_or(true);  // Default to true
        
        // Don't initialize LSP client during construction
        // It will be initialized lazily on first use
        let lsp_client = None;
        
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
        // Initialize LSP client lazily if needed
        if self.use_lsp {
            let mut lsp_guard = self.lsp_client.lock().await;
            if lsp_guard.is_none() {
                *lsp_guard = self.try_initialize_lsp().await;
            }
        }
        
        if let Some(client) = self.lsp_client.lock().await.as_mut() {
            // Ensure document is open with absolute path
            let full_path = if file_path.starts_with('/') {
                // Already absolute path
                std::path::PathBuf::from(file_path)
            } else {
                // Relative to project root
                self.project_root.join(file_path)
            };
            
            let canonical_path = full_path.canonicalize()
                .unwrap_or_else(|_| full_path.clone());
            
            let file_uri = format!("file://{}", canonical_path.to_string_lossy());
            
            let _ = client.did_open(&canonical_path.to_string_lossy()).await;
            
            let params = json!({
                "textDocument": {
                    "uri": file_uri
                },
                "position": {
                    "line": line - 1,  // LSP uses 0-based
                    "character": column - 1
                }
            });
            
            match client.hover(params).await {
                Ok(result) => {
                    // Handle different hover formats from rust-analyzer
                    if let Some(contents) = result.get("contents") {
                        // MarkedString format
                        if let Some(value) = contents.get("value").and_then(|v| v.as_str()) {
                            return Ok(Some(value.to_string()));
                        }
                        // MarkupContent format
                        else if let Some(markup) = contents.as_object() {
                            if let Some(value) = markup.get("value").and_then(|v| v.as_str()) {
                                return Ok(Some(value.to_string()));
                            }
                        }
                        // Plain string format
                        else if let Some(value) = contents.as_str() {
                            return Ok(Some(value.to_string()));
                        }
                        // Array of MarkedString
                        else if let Some(arr) = contents.as_array() {
                            let combined = arr.iter()
                                .filter_map(|item| {
                                    if let Some(s) = item.as_str() {
                                        Some(s.to_string())
                                    } else if let Some(obj) = item.as_object() {
                                        obj.get("value").and_then(|v| v.as_str()).map(|s| s.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n\n");
                            if !combined.is_empty() {
                                return Ok(Some(combined));
                            }
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
        // Initialize LSP client lazily if needed
        if self.use_lsp {
            let mut lsp_guard = self.lsp_client.lock().await;
            if lsp_guard.is_none() {
                *lsp_guard = self.try_initialize_lsp().await;
            }
        }
        
        if let Some(client) = self.lsp_client.lock().await.as_mut() {
            // Ensure document is open with absolute path
            let full_path = if file_path.starts_with('/') {
                // Already absolute path
                std::path::PathBuf::from(file_path)
            } else {
                // Relative to project root
                self.project_root.join(file_path)
            };
            
            let canonical_path = full_path.canonicalize()
                .unwrap_or_else(|_| full_path.clone());
            
            let file_uri = format!("file://{}", canonical_path.to_string_lossy());
            
            let _ = client.did_open(&canonical_path.to_string_lossy()).await;
            
            let params = json!({
                "textDocument": {
                    "uri": file_uri
                },
                "position": {
                    "line": line - 1,  // LSP uses 0-based
                    "character": column - 1
                }
            });
            
            match client.completion(params).await {
                Ok(result) => {
                    // Handle different completion response formats
                    let items = if let Some(items) = result.get("items").and_then(|v| v.as_array()) {
                        items.clone()
                    } else if result.is_array() {
                        result.as_array().unwrap().clone()
                    } else {
                        Vec::new()
                    };
                    
                    // Transform completion items to a more usable format
                    let transformed_items: Vec<Value> = items.iter().map(|item| {
                        let mut transformed = json!({
                            "label": item.get("label").and_then(|v| v.as_str()).unwrap_or(""),
                            "kind": item.get("kind").and_then(|v| v.as_u64()).unwrap_or(1),
                            "detail": item.get("detail").and_then(|v| v.as_str()).unwrap_or(""),
                            "documentation": item.get("documentation").unwrap_or(&json!(null))
                        });
                        
                        // Add insertText if available
                        if let Some(insert_text) = item.get("insertText") {
                            transformed["insertText"] = insert_text.clone();
                        }
                        
                        // Add sortText if available for proper ordering
                        if let Some(sort_text) = item.get("sortText") {
                            transformed["sortText"] = sort_text.clone();
                        }
                        
                        transformed
                    }).collect();
                    
                    Ok(transformed_items)
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
            // Ensure document is open with absolute path
            let full_path = if file_path.starts_with('/') {
                // Already absolute path
                std::path::PathBuf::from(file_path)
            } else {
                // Relative to project root
                self.project_root.join(file_path)
            };
            
            let canonical_path = full_path.canonicalize()
                .unwrap_or_else(|_| full_path.clone());
            
            let file_uri = format!("file://{}", canonical_path.to_string_lossy());
            
            let _ = client.did_open(&canonical_path.to_string_lossy()).await;
            
            let params = json!({
                "textDocument": {
                    "uri": file_uri
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
    
    pub async fn rename(&self, file: &str, line: u32, column: u32, new_name: &str) -> Result<Value> {
        if let Some(client) = self.lsp_client.lock().await.as_mut() {
            // Ensure document is open
            let full_path = if file.starts_with(&self.project_root.to_string_lossy().to_string()) {
                file.to_string()
            } else {
                self.project_root.join(file).to_string_lossy().to_string()
            };
            
            let _ = client.did_open(&full_path).await;
            
            let params = json!({
                "textDocument": {
                    "uri": format!("file://{}", full_path)
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
                    Ok(json!({
                        "error": format!("Rename failed: {}", e)
                    }))
                }
            }
        } else {
            Ok(json!({
                "error": "LSP not available"
            }))
        }
    }
    
    pub async fn signature_help(&self, file_path: &str, line: u32, column: u32) -> Result<Value> {
        if let Some(client) = self.lsp_client.lock().await.as_mut() {
            // Ensure document is open with absolute path
            let full_path = if file_path.starts_with('/') {
                // Already absolute path
                std::path::PathBuf::from(file_path)
            } else {
                // Relative to project root
                self.project_root.join(file_path)
            };
            
            let canonical_path = full_path.canonicalize()
                .unwrap_or_else(|_| full_path.clone());
            
            let file_uri = format!("file://{}", canonical_path.to_string_lossy());
            
            let _ = client.did_open(&canonical_path.to_string_lossy()).await;
            
            let params = json!({
                "textDocument": {
                    "uri": file_uri
                },
                "position": {
                    "line": line - 1,  // LSP uses 0-based
                    "character": column - 1
                }
            });
            
            match client.signature_help(params).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    info!("LSP signature help failed: {}", e);
                    Ok(json!({}))
                }
            }
        } else {
            Ok(json!({}))
        }
    }
    
    pub async fn find_implementations(&self, file_path: &str, line: u32, column: u32) -> Result<Vec<Value>> {
        if let Some(client) = self.lsp_client.lock().await.as_mut() {
            // Ensure document is open with absolute path
            let full_path = if file_path.starts_with('/') {
                // Already absolute path
                std::path::PathBuf::from(file_path)
            } else {
                // Relative to project root
                self.project_root.join(file_path)
            };
            
            let canonical_path = full_path.canonicalize()
                .unwrap_or_else(|_| full_path.clone());
            
            let file_uri = format!("file://{}", canonical_path.to_string_lossy());
            
            let _ = client.did_open(&canonical_path.to_string_lossy()).await;
            
            let params = json!({
                "textDocument": {
                    "uri": file_uri
                },
                "position": {
                    "line": line - 1,  // LSP uses 0-based
                    "character": column - 1
                }
            });
            
            match client.find_implementations(params).await {
                Ok(result) => {
                    if let Some(impls) = result.as_array() {
                        Ok(impls.clone())
                    } else {
                        Ok(Vec::new())
                    }
                }
                Err(e) => {
                    info!("LSP find implementations failed: {}", e);
                    Ok(Vec::new())
                }
            }
        } else {
            Ok(Vec::new())
        }
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
    
    pub async fn get_lsp_client(&self) -> Option<tokio::sync::MutexGuard<'_, Option<LspClient>>> {
        Some(self.lsp_client.lock().await)
    }
    
    async fn try_initialize_lsp(&self) -> Option<LspClient> {
        info!("Attempting to initialize LSP client");
        let config = LspClientConfig {
            server_path: "rust-analyzer".to_string(),
            server_args: vec![],
            root_path: self.project_root.clone(),
        };
        
        match LspClient::new(config) {
            Ok(mut client) => {
                match client.initialize().await {
                    Ok(_) => {
                        info!("LSP client initialized successfully");
                        Some(client)
                    }
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
    }
    
    pub async fn start_lsp_initialization(&self) {
        if !self.use_lsp {
            info!("LSP disabled by configuration");
            return;
        }
        
        let lsp_client = self.lsp_client.clone();
        let project_root = self.project_root.clone();
        
        // Spawn background task to initialize LSP
        tokio::spawn(async move {
            info!("Starting background LSP initialization for project: {}", project_root.display());
            let config = LspClientConfig {
                server_path: "rust-analyzer".to_string(),
                server_args: vec![],
                root_path: project_root.clone(),
            };
            
            info!("Creating LSP client...");
            match LspClient::new(config) {
                Ok(mut client) => {
                    info!("LSP client created, initializing...");
                    match client.initialize().await {
                        Ok(response) => {
                            info!("Background LSP initialization successful: {:?}", response);
                            let mut guard = lsp_client.lock().await;
                            *guard = Some(client);
                            info!("LSP client stored and ready for use");
                        }
                        Err(e) => {
                            info!("Background LSP initialization failed: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    info!("Failed to create LSP client in background: {:?}", e);
                }
            }
            info!("Background LSP initialization task completed");
        });
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