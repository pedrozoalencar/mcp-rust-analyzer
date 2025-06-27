// Test to explore jsonrpc_lite API
use jsonrpc_lite::{JsonRpc, Id, ErrorCode, Error, Request, Params};
use serde_json::json;

#[test]
fn test_jsonrpc_lite_api() {
    // Test creating an ID
    let id = Id::from(1);
    println!("ID: {:?}", id);
    
    // Test creating error codes
    let error_code = ErrorCode::from(-32600);
    println!("Error code: {:?}", error_code);
    
    // Test creating an error
    let error = Error {
        code: -32600,
        message: "Invalid request".to_string(),
        data: None,
    };
    println!("Error: {:?}", error);
    
    // Test creating a request
    let request = Request::new(
        Id::from(1),
        "test_method".to_string(),
        Some(Params::from(json!({"key": "value"})))
    );
    println!("Request: {:?}", request);
}