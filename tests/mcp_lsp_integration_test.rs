// Test MCP commands integration with LSP client
use mcp_rust_analyzer::analyzer::RustAnalyzer;
use mcp_rust_analyzer::commands::analysis::AnalysisCommands;
use mcp_rust_analyzer::server::CommandHandler;
use serde_json::json;

#[tokio::test]
async fn test_analyzer_with_lsp_backend() {
    // Given an analyzer with LSP backend
    let analyzer = RustAnalyzer::new("tests/test_project").await.unwrap();
    
    // When we analyze a symbol
    let commands = AnalysisCommands;
    let params = json!({
        "method": "analyze_symbol",
        "name": "TestStruct"
    });
    
    let result = commands.handle(Some(params), &analyzer).await;
    
    // Then it should return analysis results
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.get("symbol").is_some());
}

#[tokio::test]
async fn test_hover_with_lsp_backend() {
    // Given an analyzer with LSP backend
    let analyzer = RustAnalyzer::new("tests/test_project").await.unwrap();
    
    // When we request hover information
    let commands = AnalysisCommands;
    let params = json!({
        "method": "get_hover",
        "file": "src/lib.rs",
        "line": 3,
        "column": 12
    });
    
    let result = commands.handle(Some(params), &analyzer).await;
    
    // Then it should return hover information
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_completion_with_lsp_backend() {
    // Given an analyzer with LSP backend
    let analyzer = RustAnalyzer::new("tests/test_project").await.unwrap();
    
    // When we request completions
    let commands = mcp_rust_analyzer::commands::completion::CompletionCommands;
    let params = json!({
        "method": "complete",
        "file": "src/lib.rs",
        "line": 8,
        "column": 15
    });
    
    let result = commands.handle(Some(params), &analyzer).await;
    
    // Then it should return completion items
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.get("completions").is_some());
}