use serde_json::json;
use jsonrpc_lite::{JsonRpc, Params};

#[test]
fn test_jsonrpc_request_creation() {
    let request = jsonrpc_lite::Request::new(
        jsonrpc_lite::Id::Num(1),
        "analyze_symbol".to_string(),
        Some(Params::Map(serde_json::Map::from_iter(vec![
            ("name".to_string(), json!("test_symbol"))
        ])))
    );
    
    let json_rpc = JsonRpc::Request(request);
    let serialized = serde_json::to_string(&json_rpc).unwrap();
    
    assert!(serialized.contains("analyze_symbol"));
    assert!(serialized.contains("test_symbol"));
}

#[test]
fn test_jsonrpc_response_creation() {
    let response = JsonRpc::success(
        jsonrpc_lite::Id::Num(1),
        json!({
            "symbol": "test",
            "results": []
        })
    );
    
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("result"));
    assert!(serialized.contains("symbol"));
}

#[test]
fn test_jsonrpc_error_response() {
    let error = JsonRpc::error(
        jsonrpc_lite::Id::Num(1),
        jsonrpc_lite::Error::method_not_found()
    );
    
    let serialized = serde_json::to_string(&error).unwrap();
    assert!(serialized.contains("error"));
    assert!(serialized.contains("Method not found"));
}

#[test]
fn test_mcp_capabilities_structure() {
    let capabilities = json!({
        "name": "mcp-rust-analyzer",
        "version": "0.1.0",
        "capabilities": {
            "analysis": ["analyze_symbol", "find_references"],
            "completion": ["complete", "signature_help"],
            "refactoring": ["rename", "extract_function"],
            "metrics": ["project_structure", "code_metrics"]
        }
    });
    
    assert_eq!(capabilities["name"], "mcp-rust-analyzer");
    assert!(capabilities["capabilities"]["analysis"].is_array());
    assert!(capabilities["capabilities"]["completion"].is_array());
}