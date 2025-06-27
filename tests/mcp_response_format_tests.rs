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

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });

    let request_str = serde_json::to_string(&request)?;
    stdin.write_all(request_str.as_bytes()).await?;
    stdin.shutdown().await?;

    let mut response_buf = Vec::new();
    stdout.read_to_end(&mut response_buf).await?;
    
    child.wait().await?;

    let response_str = String::from_utf8(response_buf)?;
    for line in response_str.lines().rev() {
        if line.starts_with('{') {
            return Ok(serde_json::from_str(line)?);
        }
    }
    
    Err("No JSON response found".into())
}

#[tokio::test]
async fn test_project_structure_format() {
    let params = json!({
        "method": "project_structure"
    });
    
    let response = send_mcp_request("project_structure", params).await.unwrap();
    let result = &response["result"];
    
    // Verify root is a string path
    assert!(result["root"].is_string());
    
    // Verify type is either "package" or "workspace"
    let project_type = result["type"].as_str().unwrap();
    assert!(project_type == "package" || project_type == "workspace");
    
    // Verify modules array structure
    let modules = result["modules"].as_array().unwrap();
    for module in modules {
        // Each module must have name, path, and type
        assert!(module["name"].is_string());
        assert!(module["path"].is_string());
        assert!(module["type"].is_string());
        
        let module_type = module["type"].as_str().unwrap();
        assert!(["file", "module", "directory"].contains(&module_type));
        
        // If directory, should have submodules
        if module_type == "directory" {
            assert!(module.get("submodules").is_some());
            assert!(module["submodules"].is_array());
        }
    }
}

#[tokio::test]
async fn test_code_metrics_format() {
    let params = json!({
        "method": "code_metrics",
        "module": "src"
    });
    
    let response = send_mcp_request("code_metrics", params).await.unwrap();
    let result = &response["result"];
    
    // Verify path
    assert!(result["path"].is_string());
    assert!(result["path"].as_str().unwrap().ends_with("src"));
    
    let metrics = &result["metrics"];
    
    // Verify all numeric fields
    let numeric_fields = [
        "total_lines", "code_lines", "comment_lines", "blank_lines",
        "file_count", "functions", "structs", "enums", "traits"
    ];
    
    for field in &numeric_fields {
        assert!(metrics[field].is_u64(), "Field {} should be numeric", field);
        assert!(metrics[field].as_u64().unwrap() >= 0);
    }
    
    // Verify percentage format
    assert!(metrics["code_percentage"].is_string());
    let percentage = metrics["code_percentage"].as_str().unwrap();
    assert!(percentage.ends_with("%"));
    
    // Verify logical consistency
    let total = metrics["total_lines"].as_u64().unwrap();
    let code = metrics["code_lines"].as_u64().unwrap();
    let comments = metrics["comment_lines"].as_u64().unwrap();
    let blank = metrics["blank_lines"].as_u64().unwrap();
    assert_eq!(total, code + comments + blank);
}

#[tokio::test]
async fn test_dependencies_format() {
    let params = json!({
        "method": "analyze_dependencies"
    });
    
    let response = send_mcp_request("analyze_dependencies", params).await.unwrap();
    let result = &response["result"];
    
    // All three sections should be objects
    assert!(result["dependencies"].is_object());
    assert!(result["dev_dependencies"].is_object());
    assert!(result["build_dependencies"].is_object());
    
    // Check dependency format
    let deps = result["dependencies"].as_object().unwrap();
    for (name, version) in deps {
        assert!(name.len() > 0);
        assert!(version.is_string());
        
        // Version should be a valid format
        let version_str = version.as_str().unwrap();
        assert!(version_str.len() > 0);
    }
}

#[tokio::test]
async fn test_snippet_format() {
    let params = json!({
        "method": "expand_snippet",
        "name": "match_expr"
    });
    
    let response = send_mcp_request("expand_snippet", params).await.unwrap();
    let result = &response["result"];
    
    // Must have name and snippet
    assert_eq!(result["name"], "match_expr");
    assert!(result["snippet"].is_string());
    
    let snippet = result["snippet"].as_str().unwrap();
    
    // Verify snippet contains placeholders
    assert!(snippet.contains("${"));
    assert!(snippet.contains("}"));
    
    // Verify it's valid Rust-ish code
    assert!(snippet.contains("match"));
    assert!(snippet.contains("=>"));
}

#[tokio::test]
async fn test_hover_response_format() {
    let params = json!({
        "method": "get_hover",
        "file": "src/analyzer.rs",
        "line": 50,
        "column": 10
    });
    
    let response = send_mcp_request("get_hover", params).await.unwrap();
    let result = &response["result"];
    
    // Must return the position info
    assert_eq!(result["file"], "src/analyzer.rs");
    assert_eq!(result["line"], 50);
    assert_eq!(result["column"], 10);
    
    // Contents can be null or string/object
    assert!(result.get("contents").is_some());
}

#[tokio::test]
async fn test_completion_response_format() {
    let params = json!({
        "method": "complete",
        "file": "src/main.rs",
        "line": 10,
        "column": 15
    });
    
    let response = send_mcp_request("complete", params).await.unwrap();
    let result = &response["result"];
    
    // Must have file and position
    assert_eq!(result["file"], "src/main.rs");
    assert!(result["position"].is_object());
    assert_eq!(result["position"]["line"], 10);
    assert_eq!(result["position"]["column"], 15);
    
    // Completions must be array
    assert!(result["completions"].is_array());
    
    // If there are completions, check format
    let completions = result["completions"].as_array().unwrap();
    for completion in completions {
        // Each completion should have at least a label
        assert!(completion.get("label").is_some());
    }
}

#[tokio::test]
async fn test_references_response_format() {
    let params = json!({
        "method": "find_references",
        "file": "src/server.rs",
        "line": 25,
        "column": 10
    });
    
    let response = send_mcp_request("find_references", params).await.unwrap();
    let result = &response["result"];
    
    // Must have file and position info
    assert_eq!(result["file"], "src/server.rs");
    assert!(result["position"].is_object());
    assert_eq!(result["position"]["line"], 25);
    assert_eq!(result["position"]["column"], 10);
    
    // References must be array
    assert!(result["references"].is_array());
    
    // Check reference format if any exist
    let references = result["references"].as_array().unwrap();
    for reference in references {
        assert!(reference.get("file").is_some());
        assert!(reference.get("line").is_some());
        assert!(reference.get("column").is_some());
    }
}

#[tokio::test]
async fn test_diagnostics_response_format() {
    let params = json!({
        "method": "get_diagnostics",
        "file": "src/main.rs"
    });
    
    let response = send_mcp_request("get_diagnostics", params).await.unwrap();
    let result = &response["result"];
    
    // Must have file
    assert_eq!(result["file"], "src/main.rs");
    
    // Diagnostics must be array
    assert!(result["diagnostics"].is_array());
    
    // Currently returns pending status
    assert!(result.get("status").is_some());
    assert!(result.get("message").is_some());
}

#[tokio::test]
async fn test_improvements_response_format() {
    let params = json!({
        "method": "suggest_improvements",
        "file": "src/analyzer.rs"
    });
    
    let response = send_mcp_request("suggest_improvements", params).await.unwrap();
    let result = &response["result"];
    
    // Must have file
    assert_eq!(result["file"], "src/analyzer.rs");
    
    // Suggestions must be array
    assert!(result["suggestions"].is_array());
    
    let suggestions = result["suggestions"].as_array().unwrap();
    assert!(!suggestions.is_empty());
    
    // Check suggestion format
    for suggestion in suggestions {
        assert!(suggestion["type"].is_string());
        assert!(suggestion["message"].is_string());
        
        // Type should be meaningful
        let suggestion_type = suggestion["type"].as_str().unwrap();
        assert!(["general", "warning", "error", "info"].contains(&suggestion_type));
    }
}

#[tokio::test]
async fn test_error_response_format() {
    // Send request with missing required params
    let params = json!({
        "method": "get_hover"
        // Missing file, line, column
    });
    
    let response = send_mcp_request("get_hover", params).await.unwrap();
    
    // Must have error object
    assert!(response.get("error").is_some());
    assert!(response.get("result").is_none());
    
    let error = &response["error"];
    
    // Error must have code and message
    assert!(error["code"].is_i64());
    assert!(error["message"].is_string());
    
    // Standard JSON-RPC error codes
    let code = error["code"].as_i64().unwrap();
    assert!(code < 0); // Error codes are negative
}

#[tokio::test]
async fn test_jsonrpc_compliance() {
    let params = json!({
        "method": "project_structure"
    });
    
    let response = send_mcp_request("project_structure", params).await.unwrap();
    
    // Must have jsonrpc version
    assert_eq!(response["jsonrpc"], "2.0");
    
    // Must have id matching request
    assert_eq!(response["id"], 1);
    
    // Must have either result or error, not both
    let has_result = response.get("result").is_some();
    let has_error = response.get("error").is_some();
    assert!(has_result != has_error);
}