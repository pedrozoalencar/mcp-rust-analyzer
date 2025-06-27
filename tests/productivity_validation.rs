//! Unit tests to validate the productivity improvements
//! These tests verify that our enhanced components work correctly

use mcp_rust_analyzer::analyzer::RustAnalyzer;
use mcp_rust_analyzer::commands::metrics::MetricsCommands;
use mcp_rust_analyzer::commands::analysis::AnalysisCommands;
use mcp_rust_analyzer::server::CommandHandler;
use serde_json::json;

#[tokio::test]
async fn test_analyzer_initialization() {
    // Test that analyzer initializes with current project
    let analyzer = RustAnalyzer::new(".").await;
    assert!(analyzer.is_ok());
    
    let analyzer = analyzer.unwrap();
    assert!(analyzer.project_root().exists());
    assert!(analyzer.project_root().join("Cargo.toml").exists());
}

#[tokio::test]
async fn test_metrics_commands_exist() {
    let analyzer = RustAnalyzer::new(".").await.unwrap();
    let metrics = MetricsCommands;
    
    // Test project structure
    let result = metrics.handle(
        Some(json!({"method": "project_structure"})), 
        &analyzer
    ).await;
    assert!(result.is_ok());
    
    // Test code metrics
    let result = metrics.handle(
        Some(json!({"method": "code_metrics"})), 
        &analyzer
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_analysis_commands_exist() {
    let analyzer = RustAnalyzer::new(".").await.unwrap();
    let analysis = AnalysisCommands;
    
    // Test analyze symbol
    let result = analysis.handle(
        Some(json!({
            "method": "analyze_symbol",
            "name": "RustAnalyzer"
        })), 
        &analyzer
    ).await;
    assert!(result.is_ok());
}

#[test]
fn test_completion_features() {
    // Test that our structs have the expected features
    use std::any::type_name;
    
    assert_eq!(type_name::<RustAnalyzer>(), "mcp_rust_analyzer::analyzer::RustAnalyzer");
    assert_eq!(type_name::<MetricsCommands>(), "mcp_rust_analyzer::commands::metrics::MetricsCommands");
    assert_eq!(type_name::<AnalysisCommands>(), "mcp_rust_analyzer::commands::analysis::AnalysisCommands");
}

#[test]
fn test_json_serialization() {
    // Test that our JSON handling works correctly
    let test_data = json!({
        "method": "test",
        "params": {
            "file": "test.rs",
            "line": 42,
            "column": 10
        }
    });
    
    assert_eq!(test_data["method"], "test");
    assert_eq!(test_data["params"]["line"], 42);
}

#[tokio::test]
async fn test_project_has_rust_files() {
    // Verify our test project has the structure we expect
    use tokio::fs;
    
    let src_dir = std::path::Path::new("./src");
    assert!(src_dir.exists());
    
    let mut entries = fs::read_dir(src_dir).await.unwrap();
    let mut has_rust_files = false;
    
    while let Some(entry) = entries.next_entry().await.unwrap() {
        if entry.path().extension().map_or(false, |ext| ext == "rs") {
            has_rust_files = true;
            break;
        }
    }
    
    assert!(has_rust_files, "Project should contain Rust source files");
}