pub mod server;
pub mod analyzer;
pub mod commands;
pub mod refactor;
pub mod metrics;
pub mod lsp_client;
pub mod http_server;
pub mod http_client;
pub mod daemon_state;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_imports() {
        // Ensure all modules compile correctly
        let _ = std::hint::black_box(&server::McpServer::new);
    }
}