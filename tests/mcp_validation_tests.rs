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
async fn validate_project_structure_response() {
    let server = McpServer::new(".").await.unwrap();
    
    let response = call_mcp_method(&server, "project_structure", json!({"method": "project_structure"}))
        .await.unwrap();
    
    let result = &response["result"];
    
    // Validate root path
    assert!(result["root"].is_string());
    assert!(!result["root"].as_str().unwrap().is_empty());
    
    // Validate type
    let project_type = result["type"].as_str().unwrap();
    assert!(["package", "workspace"].contains(&project_type));
    
    // Validate modules structure
    let modules = result["modules"].as_array().unwrap();
    assert!(modules.len() > 0);
    
    // Check for expected files in our project
    let file_names: Vec<&str> = modules.iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    
    assert!(file_names.contains(&"main.rs"));
    assert!(file_names.contains(&"lib.rs"));
    assert!(file_names.contains(&"server.rs"));
    assert!(file_names.contains(&"analyzer.rs"));
    
    // Validate module entries
    for module in modules {
        assert!(module["name"].is_string());
        assert!(module["path"].is_string());
        assert!(module["type"].is_string());
        
        let module_type = module["type"].as_str().unwrap();
        assert!(["file", "module", "directory"].contains(&module_type));
        
        if module_type == "directory" {
            assert!(module["submodules"].is_array());
        }
    }
}

#[tokio::test]
async fn validate_code_metrics_response() {
    let server = McpServer::new(".").await.unwrap();
    
    let response = call_mcp_method(&server, "code_metrics", json!({
        "method": "code_metrics",
        "module": "src"
    })).await.unwrap();
    
    let result = &response["result"];
    let metrics = &result["metrics"];
    
    // Validate numeric consistency
    let total_lines = metrics["total_lines"].as_u64().unwrap();
    let code_lines = metrics["code_lines"].as_u64().unwrap();
    let comment_lines = metrics["comment_lines"].as_u64().unwrap();
    let blank_lines = metrics["blank_lines"].as_u64().unwrap();
    
    // Total should equal sum of parts
    assert_eq!(total_lines, code_lines + comment_lines + blank_lines);
    
    // Validate percentage format
    let code_percentage = metrics["code_percentage"].as_str().unwrap();
    assert!(code_percentage.ends_with("%"));
    
    // Parse percentage and validate
    let percentage_value: f64 = code_percentage
        .trim_end_matches('%')
        .parse()
        .unwrap();
    assert!(percentage_value >= 0.0 && percentage_value <= 100.0);
    
    // Validate counts are reasonable
    assert!(metrics["file_count"].as_u64().unwrap() > 5); // We have more than 5 files
    assert!(metrics["functions"].as_u64().unwrap() > 20); // We have many functions
    assert!(metrics["structs"].as_u64().unwrap() > 10); // We have several structs
}

#[tokio::test]
async fn validate_dependencies_response() {
    let server = McpServer::new(".").await.unwrap();
    
    let response = call_mcp_method(&server, "analyze_dependencies", json!({
        "method": "analyze_dependencies"
    })).await.unwrap();
    
    let result = &response["result"];
    let deps = result["dependencies"].as_object().unwrap();
    
    // Check required dependencies
    assert!(deps.contains_key("tokio"));
    assert!(deps.contains_key("serde"));
    assert!(deps.contains_key("serde_json"));
    assert!(deps.contains_key("anyhow"));
    assert!(deps.contains_key("tracing"));
    assert!(deps.contains_key("async-trait"));
    assert!(deps.contains_key("clap"));
    assert!(deps.contains_key("jsonrpc-lite"));
    assert!(deps.contains_key("tower-lsp"));
    
    // Validate version formats
    for (_name, version) in deps {
        let version_str = version.as_str().unwrap();
        assert!(!version_str.is_empty());
        // Basic version format check
        assert!(version_str.chars().any(|c| c.is_numeric() || c == '.'));
    }
    
    // Check dev dependencies
    let dev_deps = result["dev_dependencies"].as_object().unwrap();
    assert!(dev_deps.contains_key("tokio-test"));
    assert!(dev_deps.contains_key("futures"));
}

#[tokio::test]
async fn validate_snippet_expansion() {
    let server = McpServer::new(".").await.unwrap();
    
    // Test all available snippets
    let snippets = [
        ("match_expr", vec!["match", "${1:expression}", "${2:pattern}", "=>"]),
        ("if_let", vec!["if let", "${1:Some(value)}", "${2:expression}"]),
        ("for_loop", vec!["for", "${1:item}", "in", "${2:iterator}"]),
        ("impl_trait", vec!["impl", "${1:Trait}", "for", "${2:Type}"]),
        ("test_fn", vec!["#[test]", "fn", "${1:test_name}"]),
    ];
    
    for (name, expected_parts) in snippets {
        let response = call_mcp_method(&server, "expand_snippet", json!({
            "method": "expand_snippet",
            "name": name
        })).await.unwrap();
        
        let result = &response["result"];
        assert_eq!(result["name"], name);
        
        let snippet = result["snippet"].as_str().unwrap();
        
        // Verify all expected parts are present
        for part in expected_parts {
            assert!(snippet.contains(part), 
                "Snippet '{}' should contain '{}', but got: {}", name, part, snippet);
        }
        
        // Verify it's multi-line where appropriate
        if name != "test_fn" {
            assert!(snippet.contains('\n'), 
                "Snippet '{}' should be multi-line", name);
        }
    }
}

#[tokio::test]
async fn validate_error_responses() {
    let server = McpServer::new(".").await.unwrap();
    
    // Test various error conditions
    let error_cases = vec![
        // Invalid method
        ("unknown_method", json!({"method": "unknown_method"}), -32601),
        // Missing required params
        ("get_hover", json!({"method": "get_hover"}), -32603),
        ("complete", json!({"method": "complete", "file": "test.rs"}), -32603),
        // Invalid snippet name
        ("expand_snippet", json!({"method": "expand_snippet", "name": "invalid_snippet"}), -32603),
    ];
    
    for (method, params, _expected_code) in error_cases {
        let response = call_mcp_method(&server, method, params).await.unwrap();
        
        // Must have error, not result
        assert!(response.get("error").is_some());
        assert!(response.get("result").is_none());
        
        let error = &response["error"];
        assert!(error["code"].is_i64());
        assert!(error["message"].is_string());
        
        // Error code should be negative
        assert!(error["code"].as_i64().unwrap() < 0);
        
        // Message should not be empty
        assert!(!error["message"].as_str().unwrap().is_empty());
    }
}

#[tokio::test]
async fn validate_position_handling() {
    let server = McpServer::new(".").await.unwrap();
    
    // Test various position formats
    let position_methods = vec![
        ("get_hover", json!({
            "method": "get_hover",
            "file": "src/main.rs",
            "line": 1,
            "column": 1
        })),
        ("complete", json!({
            "method": "complete",
            "file": "src/lib.rs",
            "line": 10,
            "column": 20
        })),
        ("find_references", json!({
            "method": "find_references",
            "file": "src/server.rs",
            "line": 100,
            "column": 5
        })),
    ];
    
    for (method, params) in position_methods {
        let response = call_mcp_method(&server, method, params.clone()).await.unwrap();
        
        let result = &response["result"];
        
        // Response should echo back the position
        if method == "get_hover" {
            assert_eq!(result["file"], params["file"]);
            assert_eq!(result["line"], params["line"]);
            assert_eq!(result["column"], params["column"]);
        } else if method == "find_references" {
            assert_eq!(result["file"], params["file"]);
            assert_eq!(result["position"]["line"], params["line"]);
            assert_eq!(result["position"]["column"], params["column"]);
        } else {
            // Complete uses a position object
            assert_eq!(result["file"], params["file"]);
            assert_eq!(result["position"]["line"], params["line"]);
            assert_eq!(result["position"]["column"], params["column"]);
        }
    }
}

#[tokio::test]
async fn validate_file_path_handling() {
    let server = McpServer::new(".").await.unwrap();
    
    // Test various file path formats
    let file_paths = vec![
        "src/main.rs",
        "tests/test.rs",
        "./src/lib.rs",
        "src/commands/analysis.rs",
        "non_existent.rs",
        "src/../src/main.rs", // Path with ..
    ];
    
    for file_path in file_paths {
        let response = call_mcp_method(&server, "get_diagnostics", json!({
            "method": "get_diagnostics",
            "file": file_path
        })).await.unwrap();
        
        let result = &response["result"];
        
        // Should handle all paths gracefully
        assert_eq!(result["file"], file_path);
        assert!(result["diagnostics"].is_array());
    }
}

#[tokio::test]
async fn validate_large_response_handling() {
    let server = McpServer::new(".").await.unwrap();
    
    // Request metrics for entire project
    let response = call_mcp_method(&server, "code_metrics", json!({
        "method": "code_metrics",
        "module": "."
    })).await.unwrap();
    
    let result = &response["result"];
    let metrics = &result["metrics"];
    
    // Should handle large directory scan
    assert!(metrics["total_lines"].as_u64().unwrap() > 1000);
    assert!(metrics["file_count"].as_u64().unwrap() > 10);
    
    // Response should still be valid JSON
    let response_str = serde_json::to_string(&response).unwrap();
    assert!(response_str.len() > 100); // Non-trivial response
    
    // Can parse it back
    let _parsed: Value = serde_json::from_str(&response_str).unwrap();
}