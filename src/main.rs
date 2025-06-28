use anyhow::Result;
use clap::Parser;
use std::io::{self, BufRead, BufReader, Write};
use tracing::{info, error};
use tracing_subscriber;
use serde_json::json;

use mcp_rust_analyzer::server::McpServer;
use mcp_rust_analyzer::http_server::start_http_server;
use mcp_rust_analyzer::http_client::HttpClient;
use mcp_rust_analyzer::daemon_state::{DaemonState, DaemonInfo};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, help = "Project path to analyze (defaults to current directory)")]
    project_path: Option<String>,
    
    #[arg(long, default_value = "info")]
    log_level: String,
    
    #[arg(long, help = "Run as HTTP server instead of stdin/stdout")]
    server: bool,
    
    #[arg(long, help = "Port for HTTP server (auto-selected if not specified)")]
    port: Option<u16>,
    
    #[arg(long, help = "Start as daemon (background HTTP server)")]
    daemon: bool,
    
    #[arg(long, help = "Stop the daemon for current directory")]
    stop: bool,
    
    #[arg(long, help = "Check daemon status for current directory")]
    status: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&args.log_level));
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    
    info!("Starting MCP Rust Analyzer server");
    
    // Determine project path (default to current directory)
    let project_path = args.project_path.unwrap_or_else(|| ".".to_string());
    let canonical_project_path = std::fs::canonicalize(&project_path)
        .unwrap_or_else(|_| std::path::PathBuf::from(&project_path))
        .to_string_lossy()
        .to_string();
    
    info!("Project path: {}", canonical_project_path);
    
    // Handle daemon control commands
    if args.daemon {
        // Determine port (auto-select if not specified)
        let port = args.port.unwrap_or_else(|| {
            DaemonState::find_available_port().unwrap_or(3000)
        });
        
        info!("Starting daemon mode on port {} for project {}", port, canonical_project_path);
        
        // Check if daemon already exists for this project
        if let Some(existing_daemon) = DaemonState::find_daemon_for_current_dir()? {
            println!("✅ Daemon already running for this project on port {}", existing_daemon.port);
            return Ok(());
        }
        
        let http_client = HttpClient::new(port);
        http_client.start_daemon(&canonical_project_path).await?;
        
        // Register the daemon
        let mut state = DaemonState::load()?;
        state.register_daemon(&canonical_project_path, port, None)?;
        
        return Ok(());
    }
    
    if args.stop {
        info!("Stopping daemon for current directory...");
        
        if let Some(daemon_info) = DaemonState::unregister_daemon_for_current_dir()? {
            let http_client = HttpClient::new(daemon_info.port);
            http_client.stop_daemon().await?;
            println!("✅ Stopped daemon on port {}", daemon_info.port);
        } else {
            println!("❌ No daemon found for current directory");
        }
        return Ok(());
    }
    
    if args.status {
        if let Some(daemon_info) = DaemonState::find_daemon_for_current_dir()? {
            println!("✅ MCP daemon is running on port {} for project {}", 
                    daemon_info.port, daemon_info.project_path);
        } else {
            println!("❌ MCP daemon is not running for current directory");
        }
        return Ok(());
    }
    
    if args.server {
        // Direct HTTP server mode (not daemon)
        println!("🔍 [DEBUG] ENTERING SERVER MODE");
        let port = args.port.unwrap_or(3000);
        info!("Starting HTTP server mode on port {}", port);
        let server = McpServer::new(&canonical_project_path).await?;
        start_http_server(server, port).await?;
    } else {
        // Check if we're being run by Claude Code CLI (no TTY = likely MCP context)
        if !atty::is(atty::Stream::Stdin) {
            info!("Using direct mode for Claude Code CLI compatibility");
            return run_direct_mode(&canonical_project_path).await;
        }
        
        // Client mode - find daemon for current directory
        
        let daemon_info = match DaemonState::find_daemon_for_current_dir() {
            Ok(Some(daemon)) => {
                info!("Found existing daemon on port {} for project: {}", daemon.port, daemon.project_path);
                daemon
            }
            Ok(None) => {
                // No daemon found, try to auto-start one
                info!("No daemon found for current directory, attempting to start...");
                
                let port = DaemonState::find_available_port().unwrap_or(3000);
            
            let http_client = HttpClient::new(port);
            
            match http_client.start_daemon(&canonical_project_path).await {
                Ok(_) => {
                    // Register the daemon
                    let mut state = DaemonState::load()?;
                    state.register_daemon(&canonical_project_path, port, None)?;
                    
                    info!("Daemon started successfully on port {}", port);
                    DaemonInfo {
                        port,
                        project_path: canonical_project_path.clone(),
                        pid: None,
                        started_at: std::time::SystemTime::now()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)?
                            .as_secs(),
                    }
                }
                Err(e) => {
                    error!("Failed to start daemon: {}", e);
                    info!("Falling back to direct mode");
                    return run_direct_mode(&canonical_project_path).await;
                }
            }
            }
            Err(e) => {
                // State file access failed - likely permissions issue
                error!("Failed to access daemon state: {}", e);
                info!("Falling back to direct mode due to state access failure");
                return run_direct_mode(&canonical_project_path).await;
            }
        };
        
        // Client mode - forward JSON-RPC to HTTP
        info!("Running in client mode, forwarding to HTTP server on port {}", daemon_info.port);
        let http_client = HttpClient::new(daemon_info.port);
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = BufReader::new(stdin);
        
        for line in reader.lines() {
            let line = line?;
            
            match http_client.handle_jsonrpc_request(&line).await {
                Ok(response) => {
                    writeln!(stdout, "{}", response)?;
                    stdout.flush()?;
                }
                Err(e) => {
                    error!("Error forwarding request: {}", e);
                    let error_response = json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32603,
                            "message": format!("Client error: {}", e)
                        }
                    });
                    writeln!(stdout, "{}", serde_json::to_string(&error_response)?)?;
                    stdout.flush()?;
                }
            }
        }
    }
    
    Ok(())
}

async fn run_direct_mode(project_path: &str) -> Result<()> {
    info!("Running in direct mode (stdin/stdout)");
    let server = McpServer::new(project_path).await?;
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = BufReader::new(stdin);
    
    for line in reader.lines() {
        let line = line?;
        
        match server.handle_request(&line).await {
            Ok(response) => {
                writeln!(stdout, "{}", response)?;
                stdout.flush()?;
            }
            Err(e) => {
                error!("Error handling request: {}", e);
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32603,
                        "message": "Internal error"
                    }
                });
                writeln!(stdout, "{}", serde_json::to_string(&error_response)?)?;
                stdout.flush()?;
            }
        }
    }
    
    Ok(())
}