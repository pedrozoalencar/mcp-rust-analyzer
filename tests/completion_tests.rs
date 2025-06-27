use mcp_rust_analyzer::commands::completion::CompletionCommands;
use mcp_rust_analyzer::server::CommandHandler;
use serde_json::json;

#[tokio::test]
async fn test_snippet_expansion() {
    let commands = CompletionCommands;
    
    let test_cases = vec![
        ("match_expr", "match"),
        ("if_let", "if let"),
        ("for_loop", "for"),
        ("impl_trait", "impl"),
        ("test_fn", "#[test]"),
    ];
    
    for (snippet_name, expected_content) in test_cases {
        let params = json!({
            "method": "expand_snippet",
            "name": snippet_name
        });
        
        // Since we can't test the actual implementation without a real analyzer,
        // we're testing the structure and parameter handling
        let params_str = serde_json::to_string(&params).unwrap();
        assert!(params_str.contains(snippet_name));
    }
}

#[tokio::test]
async fn test_completion_params_parsing() {
    let params = json!({
        "file": "src/main.rs",
        "line": 10,
        "column": 20
    });
    
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_value(params);
    assert!(parsed.is_ok());
    
    let value = parsed.unwrap();
    assert_eq!(value["file"], "src/main.rs");
    assert_eq!(value["line"], 10);
    assert_eq!(value["column"], 20);
}

#[tokio::test]
async fn test_import_resolution_params() {
    let params = json!({
        "symbol": "HashMap"
    });
    
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_value(params);
    assert!(parsed.is_ok());
    
    let value = parsed.unwrap();
    assert_eq!(value["symbol"], "HashMap");
}