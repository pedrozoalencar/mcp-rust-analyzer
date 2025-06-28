use anyhow::{Result, Context};
use serde_json::{json, Value};
use std::time::Duration;
use tracing::{debug, warn};

pub struct HttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new(port: u16) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            base_url: format!("http://localhost:{}", port),
            client,
        }
    }
    
    pub async fn is_server_running(&self) -> bool {
        match self.client.get(&format!("{}/", self.base_url)).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
    
    pub async fn handle_jsonrpc_request(&self, request: &str) -> Result<String> {
        debug!("Forwarding JSON-RPC request to HTTP server");
        
        // Parse the request to determine the method
        let request_json: Value = serde_json::from_str(request)
            .context("Failed to parse JSON-RPC request")?;
            
        let method = request_json.get("method")
            .and_then(|m| m.as_str())
            .context("Missing method in JSON-RPC request")?;
            
        let params = request_json.get("params").cloned();
        let id = request_json.get("id").cloned();
        
        // Route to appropriate HTTP endpoint
        let response = match method {
            "initialize" => {
                self.client
                    .post(&format!("{}/initialize", self.base_url))
                    .send()
                    .await?
            }
            "tools/list" => {
                self.client
                    .get(&format!("{}/tools/list", self.base_url))
                    .send()
                    .await?
            }
            "tools/call" => {
                self.client
                    .post(&format!("{}/tools/call", self.base_url))
                    .json(&params.unwrap_or(json!({})))
                    .send()
                    .await?
            }
            "resources/list" => {
                self.client
                    .get(&format!("{}/resources/list", self.base_url))
                    .send()
                    .await?
            }
            "resources/read" => {
                self.client
                    .post(&format!("{}/resources/read", self.base_url))
                    .json(&params.unwrap_or(json!({})))
                    .send()
                    .await?
            }
            "prompts/list" => {
                self.client
                    .get(&format!("{}/prompts/list", self.base_url))
                    .send()
                    .await?
            }
            "prompts/get" => {
                self.client
                    .post(&format!("{}/prompts/get", self.base_url))
                    .json(&params.unwrap_or(json!({})))
                    .send()
                    .await?
            }
            _ => {
                // Fallback to generic JSON-RPC endpoint
                let full_request = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": method,
                    "params": params
                });
                
                self.client
                    .post(&format!("{}/jsonrpc", self.base_url))
                    .json(&full_request)
                    .send()
                    .await?
            }
        };
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("HTTP request failed with status {}: {}", status, error_text);
        }
        
        let response_text = response.text().await
            .context("Failed to read HTTP response")?;
            
        debug!("Received response from HTTP server");
        Ok(response_text)
    }
    
    pub async fn start_daemon(&self, project_path: &str) -> Result<()> {
        if self.is_server_running().await {
            warn!("Server is already running on {}", self.base_url);
            return Ok(());
        }
        
        // Extract port from base_url
        let port = self.base_url.split(':').last()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(3000);
            
        // Start the daemon process
        let mut cmd = tokio::process::Command::new(std::env::current_exe()?);
        cmd.args(&[
            "--server",
            "--port", &port.to_string(),
            "--project-path", project_path
        ]);
        
        // Spawn as daemon (detached from parent)
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        
        let child = cmd.spawn()
            .context("Failed to start daemon process")?;
            
        // Give it a moment to start
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        // Verify it's running
        for attempt in 1..=10 {
            if self.is_server_running().await {
                println!("✅ MCP server daemon started successfully on {}", self.base_url);
                return Ok(());
            }
            
            if attempt < 10 {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
        
        anyhow::bail!("Failed to start daemon - server not responding after 10 attempts");
    }
    
    pub async fn stop_daemon(&self) -> Result<()> {
        if !self.is_server_running().await {
            println!("Server is not running");
            return Ok(());
        }
        
        // For now, we'll just indicate that manual stopping is needed
        // In a production version, you might implement a shutdown endpoint
        println!("To stop the daemon, run: pkill -f 'mcp-rust-analyzer.*--server'");
        Ok(())
    }
}