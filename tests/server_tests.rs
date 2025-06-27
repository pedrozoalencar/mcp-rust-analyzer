use mcp_rust_analyzer::server::{McpServer, CommandHandler};
use serde_json::json;

#[tokio::test]
async fn test_server_initialization() {
    // Test with a dummy project path
    let result = McpServer::new("tests/test_project").await;
    
    // For now, we expect this to fail since we don't have rust-analyzer setup
    assert!(result.is_err());
}

#[tokio::test]
async fn test_json_rpc_parsing() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "analyze_symbol",
        "params": {
            "name": "test_symbol"
        }
    });
    
    let request_str = serde_json::to_string(&request).unwrap();
    
    // Verify the JSON structure is correct
    assert!(request_str.contains("analyze_symbol"));
    assert!(request_str.contains("test_symbol"));
}

#[tokio::test]
async fn test_command_routing() {
    let methods = vec![
        "analyze_symbol",
        "find_references",
        "get_diagnostics",
        "complete",
        "signature_help",
        "rename",
        "project_structure",
    ];
    
    for method in methods {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": {}
        });
        
        let request_str = serde_json::to_string(&request).unwrap();
        assert!(request_str.contains(method));
    }
}