use anyhow::{Result, Context, bail};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Command, Child};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, AsyncReadExt, BufReader, AsyncRead};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{info, debug, error};
use tokio::sync::{oneshot, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LspClientConfig {
    pub server_path: String,
    pub server_args: Vec<String>,
    pub root_path: PathBuf,
}

type ResponseMap = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>>;

pub struct LspClient {
    config: LspClientConfig,
    process: Option<Child>,
    stdin: Option<Arc<Mutex<tokio::process::ChildStdin>>>,
    request_id: Arc<AtomicU64>,
    initialized: bool,
    response_map: ResponseMap,
    _reader_handle: Option<tokio::task::JoinHandle<()>>,
}

impl LspClient {
    pub fn new(config: LspClientConfig) -> Result<Self> {
        Ok(Self {
            config,
            process: None,
            stdin: None,
            request_id: Arc::new(AtomicU64::new(1)),
            initialized: false,
            response_map: Arc::new(Mutex::new(HashMap::new())),
            _reader_handle: None,
        })
    }
    
    pub async fn initialize(&mut self) -> Result<Value> {
        // Start the LSP server process
        self.start_server().await?;
        
        // Send initialize request
        let init_params = serde_json::json!({
            "processId": std::process::id(),
            "clientInfo": {
                "name": "mcp-rust-analyzer",
                "version": "0.1.0"
            },
            "rootUri": format!("file://{}", self.config.root_path.display()),
            "capabilities": {
                "textDocument": {
                    "hover": {
                        "contentFormat": ["markdown", "plaintext"]
                    },
                    "completion": {
                        "completionItem": {
                            "snippetSupport": true
                        }
                    },
                    "references": {},
                    "rename": {
                        "prepareSupport": true
                    },
                    "signatureHelp": {
                        "signatureInformation": {
                            "documentationFormat": ["markdown", "plaintext"]
                        }
                    },
                    "implementation": {},
                    "codeAction": {}
                }
            }
        });
        
        let response = self.send_request("initialize", init_params).await?;
        self.initialized = true;
        
        // Send initialized notification
        self.send_notification("initialized", serde_json::json!({})).await?;
        
        Ok(response)
    }
    
    pub async fn shutdown(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        
        // Send shutdown request
        let _ = self.send_request("shutdown", serde_json::Value::Null).await;
        
        // Send exit notification
        let _ = self.send_notification("exit", serde_json::Value::Null).await;
        
        // Stop the reader task
        if let Some(handle) = self._reader_handle.take() {
            handle.abort();
        }
        
        // Wait for process to exit
        if let Some(mut process) = self.process.take() {
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                process.wait()
            ).await;
            
            // Force kill if still running
            let _ = process.kill().await;
        }
        
        self.initialized = false;
        Ok(())
    }
    
    pub async fn hover(&mut self, params: Value) -> Result<Value> {
        self.send_request("textDocument/hover", params).await
    }
    
    pub async fn completion(&mut self, params: Value) -> Result<Value> {
        self.send_request("textDocument/completion", params).await
    }
    
    pub async fn references(&mut self, params: Value) -> Result<Value> {
        self.send_request("textDocument/references", params).await
    }
    
    pub async fn rename(&mut self, params: Value) -> Result<Value> {
        self.send_request("textDocument/rename", params).await
    }
    
    pub async fn signature_help(&mut self, params: Value) -> Result<Value> {
        self.send_request("textDocument/signatureHelp", params).await
    }
    
    pub async fn find_implementations(&mut self, params: Value) -> Result<Value> {
        self.send_request("textDocument/implementation", params).await
    }
    
    pub async fn document_diagnostics(&mut self, params: Value) -> Result<Value> {
        self.send_request("textDocument/publishDiagnostics", params).await
    }
    
    pub async fn code_action(&mut self, params: Value) -> Result<Value> {
        self.send_request("textDocument/codeAction", params).await
    }
    
    async fn start_server(&mut self) -> Result<()> {
        info!("Starting LSP server: {}", self.config.server_path);
        
        let mut cmd = Command::new(&self.config.server_path);
        cmd.args(&self.config.server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        
        let mut process = cmd.spawn()
            .context("Failed to start LSP server")?;
        
        let stdout = process.stdout.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;
        let stdin = process.stdin.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;
        
        // Start reader task
        let response_map = self.response_map.clone();
        let reader_handle = tokio::spawn(async move {
            Self::reader_task(stdout, response_map).await;
        });
        
        self._reader_handle = Some(reader_handle);
        self.stdin = Some(Arc::new(Mutex::new(stdin)));
        self.process = Some(process);
        
        Ok(())
    }
    
    async fn reader_task(stdout: impl AsyncRead + Unpin, response_map: ResponseMap) {
        let mut reader = BufReader::new(stdout);
        let mut headers = HashMap::new();
        
        loop {
            headers.clear();
            
            // Read headers
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => return, // EOF
                    Ok(_) => {
                        let line = line.trim();
                        if line.is_empty() {
                            break; // End of headers
                        }
                        
                        if let Some((key, value)) = line.split_once(": ") {
                            headers.insert(key.to_string(), value.to_string());
                        }
                    }
                    Err(e) => {
                        error!("Error reading header: {}", e);
                        return;
                    }
                }
            }
            
            // Read content
            if let Some(content_length) = headers.get("Content-Length") {
                if let Ok(length) = content_length.parse::<usize>() {
                    let mut content = vec![0; length];
                    match reader.read_exact(&mut content).await {
                        Ok(_) => {
                            if let Ok(json) = serde_json::from_slice::<Value>(&content) {
                                debug!("Received: {}", json);
                                
                                // Handle response
                                if let Some(id) = json.get("id").and_then(|v| v.as_u64()) {
                                    let mut map = response_map.lock().await;
                                    if let Some(sender) = map.remove(&id) {
                                        if let Some(error) = json.get("error") {
                                            let _ = sender.send(Err(anyhow::anyhow!(
                                                "LSP error: {}", error
                                            )));
                                        } else if let Some(result) = json.get("result") {
                                            let _ = sender.send(Ok(result.clone()));
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error reading content: {}", e);
                            return;
                        }
                    }
                }
            }
        }
    }
    
    async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        if !self.initialized && method != "initialize" {
            bail!("LSP client not initialized");
        }
        
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        
        debug!("Sending request: {}", request);
        
        // Create response channel
        let (tx, rx) = oneshot::channel();
        {
            let mut map = self.response_map.lock().await;
            map.insert(id, tx);
        }
        
        // Send request
        self.write_message(&request).await?;
        
        // Wait for response with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(response)) => response,
            Ok(Err(_)) => bail!("Response channel closed"),
            Err(_) => {
                // Remove from map on timeout
                let mut map = self.response_map.lock().await;
                map.remove(&id);
                bail!("Request timeout")
            }
        }
    }
    
    async fn send_notification(&mut self, method: &str, params: Value) -> Result<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        
        debug!("Sending notification: {}", notification);
        self.write_message(&notification).await
    }
    
    async fn write_message(&mut self, message: &Value) -> Result<()> {
        if let Some(stdin) = &self.stdin {
            let content = serde_json::to_string(message)?;
            let header = format!("Content-Length: {}\r\n\r\n", content.len());
            
            let mut stdin_guard = stdin.lock().await;
            stdin_guard.write_all(header.as_bytes()).await?;
            stdin_guard.write_all(content.as_bytes()).await?;
            stdin_guard.flush().await?;
            
            debug!("Sent message: {}", message);
        } else {
            anyhow::bail!("No stdin available");
        }
        Ok(())
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        // Ensure process is terminated
        if let Some(mut process) = self.process.take() {
            let _ = process.start_kill();
        }
    }
}