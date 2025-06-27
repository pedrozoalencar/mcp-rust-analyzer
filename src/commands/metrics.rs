use anyhow::Result;
use serde_json::{json, Value};

use crate::analyzer::RustAnalyzer;
use crate::server::CommandHandler;

pub struct MetricsCommands;

#[async_trait::async_trait]
impl CommandHandler for MetricsCommands {
    async fn handle(&self, _params: Option<Value>, _analyzer: &RustAnalyzer) -> Result<Value> {
        // Placeholder implementation
        Ok(json!({ "status": "not implemented" }))
    }
}