// Test to understand jsonrpc_lite internals
use jsonrpc_lite::*;

#[test]
fn test_jsonrpc_structure() {
    let json_str = r#"{"jsonrpc":"2.0","id":1,"method":"test","params":{"key":"value"}}"#;
    
    match serde_json::from_str::<JsonRpc>(json_str) {
        Ok(JsonRpc::Request(req)) => {
            // Try to understand how to access fields
            let serialized = serde_json::to_string(&req).unwrap();
            println!("Serialized request: {}", serialized);
            
            // Try to create a request
            let new_req = JsonRpc::request(Id::from(1), "test");
            println!("New request: {:?}", serde_json::to_string(&new_req));
        }
        _ => panic!("Failed to parse request")
    }
}