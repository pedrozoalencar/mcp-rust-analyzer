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
async fn test_non_existent_file() {
    let params = json!({
        "method": "get_hover",
        "file": "non_existent_file.rs",
        "line": 1,
        "column": 1
    });
    
    let response = send_mcp_request("get_hover", params).await.unwrap();
    
    // Should still return a valid response
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert_eq!(result["file"], "non_existent_file.rs");
    assert!(result.get("contents").is_some());
}

#[tokio::test]
async fn test_empty_module_path() {
    let params = json!({
        "method": "code_metrics",
        "module": ""
    });
    
    let response = send_mcp_request("code_metrics", params).await.unwrap();
    
    // Should handle empty module path gracefully
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result.get("path").is_some());
    assert!(result.get("metrics").is_some());
}

#[tokio::test]
async fn test_invalid_snippet_name() {
    let params = json!({
        "method": "expand_snippet",
        "name": "invalid_snippet_name_that_does_not_exist"
    });
    
    let response = send_mcp_request("expand_snippet", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    // Should return an error message
    assert!(result.get("error").is_some());
}

#[tokio::test]
async fn test_zero_line_column() {
    let params = json!({
        "method": "get_hover",
        "file": "src/main.rs",
        "line": 0,
        "column": 0
    });
    
    let response = send_mcp_request("get_hover", params).await.unwrap();
    
    // Should handle 0-based indexing
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert_eq!(result["line"], 0);
    assert_eq!(result["column"], 0);
}

#[tokio::test]
async fn test_very_large_line_number() {
    let params = json!({
        "method": "complete",
        "file": "src/main.rs",
        "line": 999999,
        "column": 1
    });
    
    let response = send_mcp_request("complete", params).await.unwrap();
    
    // Should handle gracefully
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result["completions"].is_array());
}

#[tokio::test]
async fn test_special_characters_in_symbol() {
    let params = json!({
        "method": "analyze_symbol",
        "name": "Vec<String>"
    });
    
    let response = send_mcp_request("analyze_symbol", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert_eq!(result["symbol"], "Vec<String>");
}

#[tokio::test]
async fn test_unicode_in_filename() {
    let params = json!({
        "method": "get_diagnostics",
        "file": "src/тест_файл.rs"
    });
    
    let response = send_mcp_request("get_diagnostics", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert_eq!(result["file"], "src/тест_файл.rs");
}

#[tokio::test]
async fn test_null_optional_params() {
    let params = json!({
        "method": "code_metrics",
        "module": null
    });
    
    let response = send_mcp_request("code_metrics", params).await.unwrap();
    
    // Should use default when null
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
}

#[tokio::test]
async fn test_deeply_nested_module() {
    let params = json!({
        "method": "code_metrics",
        "module": "src/commands/analysis/deep/nested/path"
    });
    
    let response = send_mcp_request("code_metrics", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert!(result["path"].as_str().unwrap().contains("deep/nested/path"));
}

#[tokio::test]
async fn test_rename_with_same_names() {
    let params = json!({
        "method": "rename",
        "old_name": "foo",
        "new_name": "foo"
    });
    
    let response = send_mcp_request("rename", params).await.unwrap();
    
    // Should handle identical names
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
}

#[tokio::test]
async fn test_empty_string_params() {
    let params = json!({
        "method": "analyze_symbol",
        "name": ""
    });
    
    let response = send_mcp_request("analyze_symbol", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
    
    let result = &response["result"];
    assert_eq!(result["symbol"], "");
}

#[tokio::test]
async fn test_whitespace_in_params() {
    let params = json!({
        "method": "expand_snippet",
        "name": "  match_expr  "
    });
    
    let response = send_mcp_request("expand_snippet", params).await.unwrap();
    
    // Should handle whitespace
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
}

#[tokio::test]
async fn test_path_traversal_attempt() {
    let params = json!({
        "method": "get_hover",
        "file": "../../../etc/passwd",
        "line": 1,
        "column": 1
    });
    
    let response = send_mcp_request("get_hover", params).await.unwrap();
    
    // Should handle safely
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
}

#[tokio::test]
async fn test_absolute_path() {
    let params = json!({
        "method": "code_metrics",
        "module": "/absolute/path/to/module"
    });
    
    let response = send_mcp_request("code_metrics", params).await.unwrap();
    
    assert!(response["jsonrpc"] == "2.0");
    assert!(response.get("result").is_some());
}

#[tokio::test]
async fn test_concurrent_requests() {
    use futures::future::join_all;
    
    let requests = vec![
        ("project_structure", json!({"method": "project_structure"})),
        ("code_metrics", json!({"method": "code_metrics", "module": "src"})),
        ("analyze_dependencies", json!({"method": "analyze_dependencies"})),
    ];
    
    let futures: Vec<_> = requests
        .into_iter()
        .map(|(method, params)| send_mcp_request(method, params))
        .collect();
    
    let results = join_all(futures).await;
    
    // All should succeed
    for result in results {
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response["jsonrpc"] == "2.0");
        assert!(response.get("result").is_some());
    }
}