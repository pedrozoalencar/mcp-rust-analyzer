// TDD tests for RustAnalyzer integration

use mcp_rust_analyzer::analyzer::{RustAnalyzer, FileId, FilePosition, TextSize};

#[tokio::test]
async fn test_analyzer_creation_with_valid_project() {
    // Given a valid project path with Cargo.toml
    let project_path = "tests/test_project";
    
    // When creating a RustAnalyzer
    let result = RustAnalyzer::new(project_path).await;
    
    // Then it should succeed
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_analyzer_creation_without_cargo_toml() {
    // Given a path without Cargo.toml
    let project_path = "tests";
    
    // When creating a RustAnalyzer
    let result = RustAnalyzer::new(project_path).await;
    
    // Then it should fail with appropriate error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No Cargo.toml"));
}

#[tokio::test]
async fn test_get_file_id() {
    // Given a valid analyzer
    let analyzer = RustAnalyzer::new("tests/test_project").await.unwrap();
    
    // When getting a file ID
    let file_id = analyzer.get_file_id("src/lib.rs");
    
    // Then it should return a FileId
    assert!(file_id.is_ok());
}

#[tokio::test]
async fn test_get_file_position() {
    // Given a valid analyzer
    let analyzer = RustAnalyzer::new("tests/test_project").await.unwrap();
    
    // When getting a file position
    let position = analyzer.get_file_position("src/lib.rs", 10, 5);
    
    // Then it should return a FilePosition
    assert!(position.is_ok());
    let pos = position.unwrap();
    assert_eq!(pos.file_id, FileId(0)); // Stub returns FileId(0)
}

#[tokio::test]
async fn test_get_all_files() {
    // Given a valid analyzer
    let analyzer = RustAnalyzer::new("tests/test_project").await.unwrap();
    
    // When getting all files
    let files = analyzer.get_all_files();
    
    // Then it should return at least one file
    assert!(!files.is_empty());
    assert!(files[0].1.to_string_lossy().contains("src/lib.rs"));
}