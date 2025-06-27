use anyhow::Result;
use clap::Parser;
use std::io::{self, BufRead, BufReader, Write};
use tracing::{info, error};
use tracing_subscriber;

mod server;
mod analyzer;
mod commands;
mod refactor;
mod metrics;

use server::McpServer;

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
    
    tracing_subscriber::fmt()
        .with_env_filter(&args.log_level)
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
                let error_response = jsonrpc_lite::Response::error(
                    jsonrpc_lite::Id::Null,
                    jsonrpc_lite::Error::internal_error()
                );
                writeln!(stdout, "{}", serde_json::to_string(&error_response)?)?;
                stdout.flush()?;
            }
        }
    }
    
    Ok(())
}