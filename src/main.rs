use anyhow::Result;
use clap::Parser;
use std::io::{self, BufRead, BufReader, Write};
use tracing::{info, error};
use tracing_subscriber;
use serde_json::json;

use mcp_rust_analyzer::server::McpServer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = ".")]
    project_path: String,
    
    #[arg(long, default_value = "info")]
    log_level: String,
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
    info!("Project path: {}", args.project_path);
    
    let server = McpServer::new(&args.project_path).await?;
    
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