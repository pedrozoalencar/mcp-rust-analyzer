use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

async fn send_mcp_request(method: &str, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let mut child = Command::new("cargo")
        .args(&["run", "--quiet", "--"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    // Create JSON-RPC request
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });

    // Send request
    let request_str = serde_json::to_string(&request)?;
    stdin.write_all(request_str.as_bytes()).await?;
    stdin.shutdown().await?;

    // Read response
    let mut response_buf = Vec::new();
    stdout.read_to_end(&mut response_buf).await?;
    
    // Wait for process to complete
    child.wait().await?;

    // Parse response - find the JSON line
    let response_str = String::from_utf8(response_buf)?;
    for line in response_str.lines().rev() {
        if line.starts_with('{') {
            return Ok(serde_json::from_str(line)?);
        }
    }
    
    Err("No JSON response found".into())
}

#[tokio::test]
async fn test_project_structure() {
    let params = json!({
        "method": "project_structure"
    });
    
    let response = send_mcp_request("project_structure", params).await.unwrap();
    
    // Verify response structure
    assert!(response["jsonrpc"] == "2.0");
    assert!(response["id"] == 1);
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("root").is_some());
    assert!(result.get("modules").is_some());
    assert!(result.get("type").is_some());
    
    // Verify it detected our project type
    assert!(result["type"] == "package");
    
    // Verify it found some modules
    let modules = result["modules"].as_array().unwrap();
    assert!(!modules.is_empty());
    
    // Check if it found key files
    let file_names: Vec<String> = modules
        .iter()
        .map(|m| m["name"].as_str().unwrap().to_string())
        .collect();
    assert!(file_names.contains(&"main.rs".to_string()));
    assert!(file_names.contains(&"lib.rs".to_string()));
}

#[tokio::test]
async fn test_code_metrics() {
    let params = json!({
        "method": "code_metrics",
        "module": "src"
    });
    
    let response = send_mcp_request("code_metrics", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
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
    let params = json!({
        "method": "analyze_dependencies"
    });
    
    let response = send_mcp_request("analyze_dependencies", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
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
        
        let response = send_mcp_request("expand_snippet", params).await.unwrap();
        
        assert!(response["jsonrpc"] == "2.0");
        assert!(response.get("result").is_some());
        
        let result = &response["result"];
        assert!(result["name"] == name);
        assert!(result.get("snippet").is_some());
        
        let snippet = result["snippet"].as_str().unwrap();
        assert!(snippet.starts_with(expected_start), 
            "Snippet '{}' should start with '{}', but got '{}'", 
            name, expected_start, snippet);
    }
}

#[tokio::test]
async fn test_analyze_symbol() {
    let params = json!({
        "method": "analyze_symbol",
        "name": "RustAnalyzer"
    });
    
    let response = send_mcp_request("analyze_symbol", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("symbol").is_some());
    assert!(result["symbol"] == "RustAnalyzer");
    
    // Currently returns pending status
    assert!(result.get("status").is_some());
    assert!(result.get("message").is_some());
}

#[tokio::test]
async fn test_get_hover() {
    let params = json!({
        "method": "get_hover",
        "file": "src/main.rs",
        "line": 10,
        "column": 5
    });
    
    let response = send_mcp_request("get_hover", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("contents").is_some());
    assert!(result.get("file").is_some());
    assert!(result.get("line").is_some());
    assert!(result.get("column").is_some());
    
    assert!(result["file"] == "src/main.rs");
    assert!(result["line"] == 10);
    assert!(result["column"] == 5);
}

#[tokio::test]
async fn test_complete() {
    let params = json!({
        "method": "complete",
        "file": "src/main.rs",
        "line": 15,
        "column": 10
    });
    
    let response = send_mcp_request("complete", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("completions").is_some());
    assert!(result.get("file").is_some());
    assert!(result.get("position").is_some());
    
    let position = &result["position"];
    assert!(position["line"] == 15);
    assert!(position["column"] == 10);
    
    // Completions should be an array
    assert!(result["completions"].is_array());
}

#[tokio::test]
async fn test_find_references() {
    let params = json!({
        "method": "find_references",
        "file": "src/analyzer.rs",
        "line": 50,
        "column": 15
    });
    
    let response = send_mcp_request("find_references", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("references").is_some());
    assert!(result.get("file").is_some());
    assert!(result.get("position").is_some());
    
    // References should be an array
    assert!(result["references"].is_array());
}

#[tokio::test]
async fn test_get_diagnostics() {
    let params = json!({
        "method": "get_diagnostics",
        "file": "src/main.rs"
    });
    
    let response = send_mcp_request("get_diagnostics", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("diagnostics").is_some());
    assert!(result.get("file").is_some());
    
    // Diagnostics should be an array
    assert!(result["diagnostics"].is_array());
}

#[tokio::test]
async fn test_signature_help() {
    let params = json!({
        "method": "signature_help",
        "file": "src/main.rs",
        "line": 20,
        "column": 15
    });
    
    let response = send_mcp_request("signature_help", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("signatures").is_some());
    assert!(result.get("activeSignature").is_some());
    assert!(result.get("activeParameter").is_some());
    assert!(result.get("file").is_some());
    assert!(result.get("position").is_some());
    
    // Signatures should be an array
    assert!(result["signatures"].is_array());
}

#[tokio::test]
async fn test_rename() {
    let params = json!({
        "method": "rename",
        "old_name": "foo",
        "new_name": "bar"
    });
    
    let response = send_mcp_request("rename", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    // Should have status since symbol rename is not fully implemented
    assert!(result.get("status").is_some() || result.get("changes").is_some());
}

#[tokio::test]
async fn test_find_dead_code() {
    let params = json!({
        "method": "find_dead_code"
    });
    
    let response = send_mcp_request("find_dead_code", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("status").is_some());
    assert!(result.get("message").is_some());
}

#[tokio::test]
async fn test_suggest_improvements() {
    let params = json!({
        "method": "suggest_improvements",
        "file": "src/main.rs"
    });
    
    let response = send_mcp_request("suggest_improvements", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("file").is_some());
    assert!(result.get("suggestions").is_some());
    
    assert!(result["file"] == "src/main.rs");
    
    // Suggestions should be an array
    let suggestions = result["suggestions"].as_array().unwrap();
    assert!(!suggestions.is_empty());
    
    // Each suggestion should have type and message
    for suggestion in suggestions {
        assert!(suggestion.get("type").is_some());
        assert!(suggestion.get("message").is_some());
    }
}

#[tokio::test]
async fn test_error_handling() {
    // Test with invalid method
    let params = json!({
        "method": "invalid_method"
    });
    
    let response = send_mcp_request("invalid_method", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("error").is_some());
    
    let error = &response["error"];
    assert!(error.get("code").is_some());
    assert!(error.get("message").is_some());
    assert_eq!(error["code"], -32601); // Method not found
}

#[tokio::test]
async fn test_missing_params() {
    // Test complete without required params
    let params = json!({
        "method": "complete"
        // Missing file, line, column
    });
    
    let response = send_mcp_request("complete", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("error").is_some());
    
    let error = &response["error"];
    assert!(error.get("code").is_some());
    assert!(error.get("message").is_some());
}