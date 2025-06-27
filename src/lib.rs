pub mod server;
pub mod analyzer;
pub mod commands;
pub mod refactor;
pub mod metrics;
pub mod lsp_client;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_imports() {
        // Ensure all modules compile correctly
        let _ = std::hint::black_box(&server::McpServer::new);
    }
}