// Integration test with real rust-analyzer process
use mcp_rust_analyzer::lsp_client::{LspClient, LspClientConfig};
use std::path::PathBuf;

#[tokio::test] 
#[ignore] // Run with: cargo test real_lsp_integration_test -- --ignored
async fn test_real_rust_analyzer_integration() {
    // Check if rust-analyzer is available
    let output = std::process::Command::new("rust-analyzer")
        .arg("--version")
        .output();
        
    if output.is_err() {
        eprintln!("rust-analyzer not found in PATH. Skipping test.");
        return;
    }
    
    // Given a real rust-analyzer process
    let config = LspClientConfig {
        server_path: "rust-analyzer".to_string(),
        server_args: vec![],
        root_path: PathBuf::from("tests/test_project").canonicalize().unwrap(),
    };
    
    let mut client = LspClient::new(config).unwrap();
    
    // When initializing
    let result = client.initialize().await;
    
    // Then it should return capabilities
    assert!(result.is_ok());
    let capabilities = result.unwrap();
    println!("Server capabilities: {:#?}", capabilities);
    
    // And we should be able to shutdown
    let shutdown_result = client.shutdown().await;
    assert!(shutdown_result.is_ok());
}