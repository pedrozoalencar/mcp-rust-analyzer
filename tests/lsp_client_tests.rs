// TDD tests for LSP client integration with rust-analyzer

use mcp_rust_analyzer::lsp_client::{LspClient, LspClientConfig};
use serde_json::json;
use std::path::PathBuf;

#[tokio::test]
async fn test_lsp_client_creation() {
    // Given a configuration for LSP client
    let config = LspClientConfig {
        server_path: "rust-analyzer".to_string(),
        server_args: vec![],
        root_path: PathBuf::from("tests/test_project"),
    };
    
    // When creating an LSP client
    let client = LspClient::new(config);
    
    // Then it should be created successfully
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_lsp_client_initialize() {
    // Given an LSP client
    let config = LspClientConfig {
        server_path: "rust-analyzer".to_string(),
        server_args: vec![],
        root_path: PathBuf::from("tests/test_project"),
    };
    
    let mut client = LspClient::new(config).unwrap();
    
    // When initializing the client
    let result = client.initialize().await;
    
    // Then it should return server capabilities
    assert!(result.is_ok());
    let capabilities = result.unwrap();
    assert!(capabilities.is_object());
}

#[tokio::test]
async fn test_lsp_client_shutdown() {
    // Given an initialized LSP client
    let config = LspClientConfig {
        server_path: "rust-analyzer".to_string(),
        server_args: vec![],
        root_path: PathBuf::from("tests/test_project"),
    };
    
    let mut client = LspClient::new(config).unwrap();
    let _ = client.initialize().await;
    
    // When shutting down the client
    let result = client.shutdown().await;
    
    // Then it should shutdown gracefully
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_lsp_text_document_hover() {
    // Given an initialized LSP client and a file position
    let config = LspClientConfig {
        server_path: "rust-analyzer".to_string(),
        server_args: vec![],
        root_path: PathBuf::from("tests/test_project"),
    };
    
    let mut client = LspClient::new(config).unwrap();
    let _ = client.initialize().await;
    
    // When requesting hover information
    let hover_params = json!({
        "textDocument": {
            "uri": "file://tests/test_project/src/lib.rs"
        },
        "position": {
            "line": 2,
            "character": 10
        }
    });
    
    let result = client.hover(hover_params).await;
    
    // Then it should return hover information
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_lsp_text_document_completion() {
    // Given an initialized LSP client and a position
    let config = LspClientConfig {
        server_path: "rust-analyzer".to_string(),
        server_args: vec![],
        root_path: PathBuf::from("tests/test_project"),
    };
    
    let mut client = LspClient::new(config).unwrap();
    let _ = client.initialize().await;
    
    // When requesting completions
    let completion_params = json!({
        "textDocument": {
            "uri": "file://tests/test_project/src/lib.rs"
        },
        "position": {
            "line": 8,
            "character": 15
        }
    });
    
    let result = client.completion(completion_params).await;
    
    // Then it should return completion items
    assert!(result.is_ok());
    let completions = result.unwrap();
    assert!(completions.is_array() || completions.get("items").is_some());
}

#[tokio::test]
async fn test_lsp_find_references() {
    // Given an initialized LSP client
    let config = LspClientConfig {
        server_path: "rust-analyzer".to_string(),
        server_args: vec![],
        root_path: PathBuf::from("tests/test_project"),
    };
    
    let mut client = LspClient::new(config).unwrap();
    let _ = client.initialize().await;
    
    // When finding references
    let reference_params = json!({
        "textDocument": {
            "uri": "file://tests/test_project/src/lib.rs"
        },
        "position": {
            "line": 2,
            "character": 10
        },
        "context": {
            "includeDeclaration": true
        }
    });
    
    let result = client.references(reference_params).await;
    
    // Then it should return a list of references
    assert!(result.is_ok());
}

#[tokio::test] 
async fn test_lsp_rename() {
    // Given an initialized LSP client
    let config = LspClientConfig {
        server_path: "rust-analyzer".to_string(),
        server_args: vec![],
        root_path: PathBuf::from("tests/test_project"),
    };
    
    let mut client = LspClient::new(config).unwrap();
    let _ = client.initialize().await;
    
    // When renaming a symbol
    let rename_params = json!({
        "textDocument": {
            "uri": "file://tests/test_project/src/lib.rs"
        },
        "position": {
            "line": 2,
            "character": 10
        },
        "newName": "RenamedStruct"
    });
    
    let result = client.rename(rename_params).await;
    
    // Then it should return workspace edits
    assert!(result.is_ok());
    let edits = result.unwrap();
    assert!(edits.get("changes").is_some() || edits.get("documentChanges").is_some());
}