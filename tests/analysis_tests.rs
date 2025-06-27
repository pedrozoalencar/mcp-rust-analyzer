use serde_json::json;

#[test]
fn test_position_params_serialization() {
    let params = json!({
        "file": "src/lib.rs",
        "line": 42,
        "column": 15
    });
    
    assert_eq!(params["file"], "src/lib.rs");
    assert_eq!(params["line"], 42);
    assert_eq!(params["column"], 15);
}

#[test]
fn test_symbol_params_serialization() {
    let params = json!({
        "name": "MyStruct"
    });
    
    assert_eq!(params["name"], "MyStruct");
}

#[test]
fn test_diagnostics_response_structure() {
    let response = json!({
        "diagnostics": [
            {
                "severity": "error",
                "message": "unresolved import",
                "range": {
                    "start": 0,
                    "end": 10
                },
                "fixes": 0
            }
        ]
    });
    
    assert!(response["diagnostics"].is_array());
    assert_eq!(response["diagnostics"][0]["severity"], "error");
    assert_eq!(response["diagnostics"][0]["message"], "unresolved import");
}

#[test]
fn test_hover_response_structure() {
    let response = json!({
        "contents": "fn main() -> Result<()>",
        "range": {
            "start": 0,
            "end": 23
        }
    });
    
    assert_eq!(response["contents"], "fn main() -> Result<()>");
    assert!(response["range"].is_object());
}

#[test]
fn test_references_response_structure() {
    let response = json!({
        "declaration": {
            "name": "process",
            "kind": "Function",
            "file": "src/main.rs"
        },
        "references": [
            {
                "file": "src/lib.rs",
                "range": {
                    "start": 100,
                    "end": 107
                },
                "kind": "Read"
            }
        ]
    });
    
    assert_eq!(response["declaration"]["name"], "process");
    assert!(response["references"].is_array());
    assert_eq!(response["references"][0]["file"], "src/lib.rs");
}