use mcp_rust_analyzer::server::McpServer;
use serde_json::{json, Value};

async fn call_mcp_method(server: &McpServer, method: &str, params: Value) -> Result<Value, String> {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });
    
    let request_str = serde_json::to_string(&request).unwrap();
    let response_str = server.handle_request(&request_str).await
        .map_err(|e| e.to_string())?;
    
    serde_json::from_str(&response_str)
        .map_err(|e| e.to_string())
}

#[tokio::test]
async fn test_project_structure() {
    let server = McpServer::new(".").await.unwrap();
    
    let params = json!({
        "method": "project_structure"
    });
    
    let response = call_mcp_method(&server, "project_structure", params).await.unwrap();
    
    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("root").is_some());
    assert!(result.get("modules").is_some());
    assert!(result.get("type").is_some());
    
    // Verify it detected our project type
    assert_eq!(result["type"], "package");
    
    // Verify it found some modules
    let modules = result["modules"].as_array().unwrap();
    assert!(!modules.is_empty());
}

#[tokio::test]
async fn test_code_metrics() {
    let server = McpServer::new(".").await.unwrap();
    
    let params = json!({
        "method": "code_metrics",
        "module": "src"
    });
    
    let response = call_mcp_method(&server, "code_metrics", params).await.unwrap();
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("path").is_some());
    assert!(result.get("metrics").is_some());
    
    let metrics = &result["metrics"];
    // Verify all expected metric fields
    assert!(metrics.get("total_lines").is_some());
    assert!(metrics.get("code_lines").is_some());
    assert!(metrics.get("comment_lines").is_some());
    assert!(metrics.get("blank_lines").is_some());
    assert!(metrics.get("file_count").is_some());
    assert!(metrics.get("functions").is_some());
    assert!(metrics.get("structs").is_some());
    assert!(metrics.get("enums").is_some());
    assert!(metrics.get("traits").is_some());
    assert!(metrics.get("code_percentage").is_some());
    
    // Verify metrics are reasonable
    assert!(metrics["total_lines"].as_u64().unwrap() > 0);
    assert!(metrics["code_lines"].as_u64().unwrap() > 0);
    assert!(metrics["file_count"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn test_analyze_dependencies() {
    let server = McpServer::new(".").await.unwrap();
    
    let params = json!({
        "method": "analyze_dependencies"
    });
    
    let response = call_mcp_method(&server, "analyze_dependencies", params).await.unwrap();
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("dependencies").is_some());
    assert!(result.get("dev_dependencies").is_some());
    assert!(result.get("build_dependencies").is_some());
    
    // Verify we have some dependencies
    let deps = result["dependencies"].as_object().unwrap();
    assert!(!deps.is_empty());
    
    // Check for known dependencies
    assert!(deps.contains_key("tokio"));
    assert!(deps.contains_key("serde"));
    assert!(deps.contains_key("serde_json"));
}

#[tokio::test]
async fn test_expand_snippet() {
    let server = McpServer::new(".").await.unwrap();
    
    let test_snippets = vec![
        ("match_expr", "match ${1:expression}"),
        ("if_let", "if let ${1:Some(value)}"),
        ("for_loop", "for ${1:item} in ${2:iterator}"),
        ("impl_trait", "impl ${1:Trait} for ${2:Type}"),
        ("test_fn", "#[test]"),
    ];
    
    for (name, expected_start) in test_snippets {
        let params = json!({
            "method": "expand_snippet",
            "name": name
        });
        
        let response = call_mcp_method(&server, "expand_snippet", params).await.unwrap();
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response.get("result").is_some());
        
        let result = &response["result"];
        assert_eq!(result["name"], name);
        assert!(result.get("snippet").is_some());
        
        let snippet = result["snippet"].as_str().unwrap();
        assert!(snippet.starts_with(expected_start), 
            "Snippet '{}' should start with '{}', but got '{}'", 
            name, expected_start, snippet);
    }
}

#[tokio::test]
async fn test_analyze_symbol() {
    let server = McpServer::new(".").await.unwrap();
    
    let params = json!({
        "method": "analyze_symbol",
        "name": "RustAnalyzer"
    });
    
    let response = call_mcp_method(&server, "analyze_symbol", params).await.unwrap();
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("symbol").is_some());
    assert_eq!(result["symbol"], "RustAnalyzer");
    
    // Currently returns pending status
    assert!(result.get("status").is_some());
    assert!(result.get("message").is_some());
}

#[tokio::test]
async fn test_get_hover() {
    let server = McpServer::new(".").await.unwrap();
    
    let params = json!({
        "method": "get_hover",
        "file": "src/main.rs",
        "line": 10,
        "column": 5
    });
    
    let response = call_mcp_method(&server, "get_hover", params).await.unwrap();
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("contents").is_some());
    assert!(result.get("file").is_some());
    assert!(result.get("line").is_some());
    assert!(result.get("column").is_some());
    
    assert_eq!(result["file"], "src/main.rs");
    assert_eq!(result["line"], 10);
    assert_eq!(result["column"], 5);
}

#[tokio::test]
async fn test_complete() {
    let server = McpServer::new(".").await.unwrap();
    
    let params = json!({
        "method": "complete",
        "file": "src/main.rs",
        "line": 15,
        "column": 10
    });
    
    let response = call_mcp_method(&server, "complete", params).await.unwrap();
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("completions").is_some());
    assert!(result.get("file").is_some());
    assert!(result.get("position").is_some());
    
    let position = &result["position"];
    assert_eq!(position["line"], 15);
    assert_eq!(position["column"], 10);
    
    // Completions should be an array
    assert!(result["completions"].is_array());
}

#[tokio::test]
async fn test_find_references() {
    let server = McpServer::new(".").await.unwrap();
    
    let params = json!({
        "method": "find_references",
        "file": "src/analyzer.rs",
        "line": 50,
        "column": 15
    });
    
    let response = call_mcp_method(&server, "find_references", params).await.unwrap();
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("references").is_some());
    assert!(result.get("file").is_some());
    assert!(result.get("position").is_some());
    
    // References should be an array
    assert!(result["references"].is_array());
}

#[tokio::test]
async fn test_error_handling() {
    let server = McpServer::new(".").await.unwrap();
    
    // Test with invalid method
    let params = json!({
        "method": "invalid_method"
    });
    
    let response = call_mcp_method(&server, "invalid_method", params).await.unwrap();
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("error").is_some());
    
    let error = &response["error"];
    assert!(error.get("code").is_some());
    assert!(error.get("message").is_some());
    assert_eq!(error["code"], -32601); // Method not found
}

#[tokio::test]
async fn test_missing_params() {
    let server = McpServer::new(".").await.unwrap();
    
    // Test complete without required params
    let params = json!({
        "method": "complete"
        // Missing file, line, column
    });
    
    let response = call_mcp_method(&server, "complete", params).await.unwrap();
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("error").is_some());
    
    let error = &response["error"];
    assert!(error.get("code").is_some());
    assert!(error.get("message").is_some());
}

#[tokio::test]
async fn test_response_format_consistency() {
    let server = McpServer::new(".").await.unwrap();
    
    // Test multiple methods for consistent response format
    let test_cases = vec![
        ("project_structure", json!({"method": "project_structure"})),
        ("code_metrics", json!({"method": "code_metrics", "module": "src"})),
        ("analyze_dependencies", json!({"method": "analyze_dependencies"})),
    ];
    
    for (method, params) in test_cases {
        let response = call_mcp_method(&server, method, params).await.unwrap();
        
        // All responses must have these fields
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        
        // Must have either result or error, not both
        let has_result = response.get("result").is_some();
        let has_error = response.get("error").is_some();
        assert!(has_result != has_error, 
            "Response for {} must have either result or error, not both", method);
    }
}

#[tokio::test]
async fn test_edge_cases() {
    let server = McpServer::new(".").await.unwrap();
    
    // Test with empty string
    let params = json!({
        "method": "analyze_symbol",
        "name": ""
    });
    
    let response = call_mcp_method(&server, "analyze_symbol", params).await.unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    
    // Test with special characters
    let params = json!({
        "method": "analyze_symbol",
        "name": "Vec<String>"
    });
    
    let response = call_mcp_method(&server, "analyze_symbol", params).await.unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
    
    // Test with very large line number
    let params = json!({
        "method": "get_hover",
        "file": "src/main.rs",
        "line": 999999,
        "column": 1
    });
    
    let response = call_mcp_method(&server, "get_hover", params).await.unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("result").is_some());
}