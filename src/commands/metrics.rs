use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::debug;
use std::path::Path;
use tokio::fs;

use crate::analyzer::RustAnalyzer;
use crate::server::CommandHandler;

#[derive(Debug, Serialize, Deserialize)]
struct ModuleParams {
    module: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileParams {
    file: String,
}

pub struct MetricsCommands;

#[async_trait::async_trait]
impl CommandHandler for MetricsCommands {
    async fn handle(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let method = params
            .as_ref()
            .and_then(|p| p.get("method"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
            
        match method {
            "project_structure" => self.project_structure(analyzer).await,
            "analyze_dependencies" => self.analyze_dependencies(analyzer).await,
            "code_metrics" => self.code_metrics(params, analyzer).await,
            "find_dead_code" => self.find_dead_code(analyzer).await,
            "suggest_improvements" => self.suggest_improvements(params, analyzer).await,
            _ => anyhow::bail!("Unknown metrics method: {}", method),
        }
    }
}

impl MetricsCommands {
    async fn project_structure(&self, analyzer: &RustAnalyzer) -> Result<Value> {
        debug!("Analyzing project structure");
        
        let mut structure = json!({
            "root": analyzer.project_root().display().to_string(),
            "modules": []
        });
        
        // Analyze src directory structure
        if let Ok(modules) = self.analyze_directory(&analyzer.project_root().join("src")).await {
            structure["modules"] = json!(modules);
        }
        
        // Check for workspace
        let cargo_toml = analyzer.project_root().join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml).await {
                if content.contains("[workspace]") {
                    structure["type"] = json!("workspace");
                    structure["members"] = json!(self.find_workspace_members(analyzer.project_root()).await?);
                } else {
                    structure["type"] = json!("package");
                }
            }
        }
        
        Ok(structure)
    }
    
    async fn analyze_directory(&self, path: &Path) -> Result<Vec<Value>> {
        let mut modules = Vec::new();
        
        if !path.exists() {
            return Ok(modules);
        }
        
        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            if path.is_dir() && !file_name.starts_with('.') {
                let submodules = Box::pin(self.analyze_directory(&path)).await?;
                modules.push(json!({
                    "name": file_name,
                    "type": "directory",
                    "path": path.display().to_string(),
                    "submodules": submodules
                }));
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                let is_mod = file_name == "mod.rs" || file_name == "lib.rs" || file_name == "main.rs";
                modules.push(json!({
                    "name": file_name,
                    "type": if is_mod { "module" } else { "file" },
                    "path": path.display().to_string()
                }));
            }
        }
        
        Ok(modules)
    }
    
    async fn find_workspace_members(&self, root: &Path) -> Result<Vec<String>> {
        let mut members = Vec::new();
        
        let cargo_toml = root.join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(cargo_toml).await {
            // Simple parsing - in real implementation would use toml crate
            for line in content.lines() {
                if line.trim().starts_with("members") {
                    // Extract members from the line or following array
                    members.push("(workspace member detection not fully implemented)".to_string());
                    break;
                }
            }
        }
        
        Ok(members)
    }
    
    async fn analyze_dependencies(&self, analyzer: &RustAnalyzer) -> Result<Value> {
        debug!("Analyzing dependencies");
        
        let cargo_toml = analyzer.project_root().join("Cargo.toml");
        let mut deps = json!({
            "dependencies": {},
            "dev_dependencies": {},
            "build_dependencies": {}
        });
        
        if let Ok(content) = fs::read_to_string(cargo_toml).await {
            // Simple parsing - in real implementation would use toml crate
            let mut current_section = "";
            
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == "[dependencies]" {
                    current_section = "dependencies";
                } else if trimmed == "[dev-dependencies]" {
                    current_section = "dev_dependencies";
                } else if trimmed == "[build-dependencies]" {
                    current_section = "build_dependencies";
                } else if trimmed.starts_with('[') {
                    current_section = "";
                } else if !current_section.is_empty() && trimmed.contains('=') {
                    if let Some((name, version)) = trimmed.split_once('=') {
                        let name = name.trim();
                        let version = version.trim().trim_matches('"');
                        deps[current_section][name] = json!(version);
                    }
                }
            }
        }
        
        Ok(deps)
    }
    
    async fn code_metrics(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params: ModuleParams = serde_json::from_value(
            params.unwrap_or_else(|| json!({}))
        )?;
        
        debug!("Calculating code metrics for module: {:?}", params.module);
        
        let target_path = if let Some(module) = params.module {
            analyzer.project_root().join(module)
        } else {
            analyzer.project_root().join("src")
        };
        
        let metrics = self.calculate_metrics(&target_path).await?;
        
        Ok(json!({
            "path": target_path.display().to_string(),
            "metrics": metrics
        }))
    }
    
    async fn calculate_metrics(&self, path: &Path) -> Result<Value> {
        let mut total_lines = 0u64;
        let mut code_lines = 0u64;
        let mut comment_lines = 0u64;
        let mut blank_lines = 0u64;
        let mut file_count = 0u64;
        let mut function_count = 0u64;
        let mut struct_count = 0u64;
        let mut enum_count = 0u64;
        let mut trait_count = 0u64;
        
        if path.is_file() {
            if path.extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(path).await?;
                let stats = self.analyze_file_content(&content);
                return Ok(stats);
            }
        } else if path.is_dir() {
            let mut entries = fs::read_dir(path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                    file_count += 1;
                    if let Ok(content) = fs::read_to_string(&path).await {
                        let stats = self.analyze_file_content(&content);
                        if let Some(lines) = stats.get("total_lines").and_then(|v| v.as_u64()) {
                            total_lines += lines;
                        }
                        if let Some(lines) = stats.get("code_lines").and_then(|v| v.as_u64()) {
                            code_lines += lines;
                        }
                        if let Some(lines) = stats.get("comment_lines").and_then(|v| v.as_u64()) {
                            comment_lines += lines;
                        }
                        if let Some(lines) = stats.get("blank_lines").and_then(|v| v.as_u64()) {
                            blank_lines += lines;
                        }
                        if let Some(count) = stats.get("functions").and_then(|v| v.as_u64()) {
                            function_count += count;
                        }
                        if let Some(count) = stats.get("structs").and_then(|v| v.as_u64()) {
                            struct_count += count;
                        }
                        if let Some(count) = stats.get("enums").and_then(|v| v.as_u64()) {
                            enum_count += count;
                        }
                        if let Some(count) = stats.get("traits").and_then(|v| v.as_u64()) {
                            trait_count += count;
                        }
                    }
                } else if path.is_dir() && !entry.file_name().to_string_lossy().starts_with('.') {
                    // Recursively analyze subdirectories
                    if let Ok(submetrics) = Box::pin(self.calculate_metrics(&path)).await {
                        if let Some(count) = submetrics.get("file_count").and_then(|v| v.as_u64()) {
                            file_count += count;
                        }
                        if let Some(lines) = submetrics.get("total_lines").and_then(|v| v.as_u64()) {
                            total_lines += lines;
                        }
                        if let Some(lines) = submetrics.get("code_lines").and_then(|v| v.as_u64()) {
                            code_lines += lines;
                        }
                        if let Some(lines) = submetrics.get("comment_lines").and_then(|v| v.as_u64()) {
                            comment_lines += lines;
                        }
                        if let Some(lines) = submetrics.get("blank_lines").and_then(|v| v.as_u64()) {
                            blank_lines += lines;
                        }
                        if let Some(count) = submetrics.get("functions").and_then(|v| v.as_u64()) {
                            function_count += count;
                        }
                        if let Some(count) = submetrics.get("structs").and_then(|v| v.as_u64()) {
                            struct_count += count;
                        }
                        if let Some(count) = submetrics.get("enums").and_then(|v| v.as_u64()) {
                            enum_count += count;
                        }
                        if let Some(count) = submetrics.get("traits").and_then(|v| v.as_u64()) {
                            trait_count += count;
                        }
                    }
                }
            }
        }
        
        Ok(json!({
            "file_count": file_count,
            "total_lines": total_lines,
            "code_lines": code_lines,
            "comment_lines": comment_lines,
            "blank_lines": blank_lines,
            "code_percentage": if total_lines > 0 { 
                format!("{:.1}%", (code_lines as f64 / total_lines as f64) * 100.0) 
            } else { 
                "0.0%".to_string() 
            },
            "functions": function_count,
            "structs": struct_count,
            "enums": enum_count,
            "traits": trait_count
        }))
    }
    
    fn analyze_file_content(&self, content: &str) -> Value {
        let mut total_lines = 0;
        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut blank_lines = 0;
        let mut in_block_comment = false;
        let mut function_count = 0;
        let mut struct_count = 0;
        let mut enum_count = 0;
        let mut trait_count = 0;
        
        for line in content.lines() {
            total_lines += 1;
            let trimmed = line.trim();
            
            if in_block_comment {
                comment_lines += 1;
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
            } else if trimmed.starts_with("/*") {
                comment_lines += 1;
                if !trimmed.contains("*/") {
                    in_block_comment = true;
                }
            } else if trimmed.starts_with("//") {
                comment_lines += 1;
            } else if trimmed.is_empty() {
                blank_lines += 1;
            } else {
                code_lines += 1;
                
                // Simple pattern matching for declarations
                if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") ||
                   trimmed.starts_with("async fn ") || trimmed.starts_with("pub async fn ") {
                    function_count += 1;
                } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                    struct_count += 1;
                } else if trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ") {
                    enum_count += 1;
                } else if trimmed.starts_with("trait ") || trimmed.starts_with("pub trait ") {
                    trait_count += 1;
                }
            }
        }
        
        json!({
            "total_lines": total_lines,
            "code_lines": code_lines,
            "comment_lines": comment_lines,
            "blank_lines": blank_lines,
            "functions": function_count,
            "structs": struct_count,
            "enums": enum_count,
            "traits": trait_count
        })
    }
    
    async fn find_dead_code(&self, analyzer: &RustAnalyzer) -> Result<Value> {
        debug!("Finding dead code");
        
        use tokio::process::Command;
        
        // Run cargo check with dead code detection
        let output = Command::new("cargo")
            .args(&["check", "--all-targets", "--message-format=json"])
            .current_dir(analyzer.project_root())
            .env("RUSTFLAGS", "-W dead_code")
            .output()
            .await;
            
        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                
                let mut dead_code_warnings = Vec::new();
                
                // Parse cargo output for dead code warnings
                for line in stdout.lines() {
                    if let Ok(json_msg) = serde_json::from_str::<Value>(line) {
                        if json_msg.get("reason") == Some(&json!("compiler-message")) {
                            if let Some(message) = json_msg.get("message") {
                                if let Some(code) = message.get("code") {
                                    if code.get("code") == Some(&json!("dead_code")) {
                                        dead_code_warnings.push(json!({
                                            "level": message.get("level").unwrap_or(&json!("warning")),
                                            "message": message.get("message").unwrap_or(&json!("")),
                                            "spans": message.get("spans").unwrap_or(&json!([]))
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
                
                Ok(json!({
                    "dead_code_warnings": dead_code_warnings,
                    "total_warnings": dead_code_warnings.len(),
                    "success": result.status.success(),
                    "stderr": if !stderr.is_empty() { Some(stderr) } else { None }
                }))
            }
            Err(e) => Ok(json!({
                "error": format!("Failed to run cargo check: {}", e),
                "hint": "Ensure cargo is installed and project has valid Cargo.toml"
            }))
        }
    }
    
    async fn suggest_improvements(&self, params: Option<Value>, analyzer: &RustAnalyzer) -> Result<Value> {
        let params: FileParams = serde_json::from_value(
            params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?
        )?;
        
        debug!("Suggesting improvements for {}", params.file);
        
        use tokio::process::Command;
        
        let mut suggestions = Vec::new();
        
        // Run clippy for specific file
        let clippy_output = Command::new("cargo")
            .args(&["clippy", "--message-format=json", "--", "-A", "clippy::all"])
            .current_dir(analyzer.project_root())
            .output()
            .await;
            
        match clippy_output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                
                // Parse clippy output for suggestions
                for line in stdout.lines() {
                    if let Ok(json_msg) = serde_json::from_str::<Value>(line) {
                        if json_msg.get("reason") == Some(&json!("compiler-message")) {
                            if let Some(message) = json_msg.get("message") {
                                if let Some(spans) = message.get("spans").and_then(|s| s.as_array()) {
                                    for span in spans {
                                        if let Some(file_name) = span.get("file_name").and_then(|f| f.as_str()) {
                                            if file_name.contains(&params.file) || params.file.contains(file_name) {
                                                suggestions.push(json!({
                                                    "type": "clippy",
                                                    "level": message.get("level").unwrap_or(&json!("suggestion")),
                                                    "message": message.get("message").unwrap_or(&json!("")),
                                                    "code": message.get("code"),
                                                    "line": span.get("line_start"),
                                                    "column": span.get("column_start"),
                                                    "suggestion": span.get("suggested_replacement")
                                                }));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // If clippy fails, provide general suggestions
                suggestions.push(json!({
                    "type": "general",
                    "message": "Install clippy with 'rustup component add clippy' for detailed suggestions"
                }));
            }
        }
        
        // Add general improvement suggestions
        suggestions.extend([
            json!({
                "type": "formatting",
                "message": "Run 'cargo fmt' to ensure consistent formatting",
                "command": "cargo fmt"
            }),
            json!({
                "type": "documentation",
                "message": "Consider adding documentation comments for public items",
                "example": "/// Description of the function\npub fn example() {}"
            }),
            json!({
                "type": "testing",
                "message": "Add unit tests for critical functions",
                "example": "#[cfg(test)]\nmod tests {\n    #[test]\n    fn test_function() {}\n}"
            })
        ]);
        
        Ok(json!({
            "file": params.file,
            "suggestions": suggestions,
            "total_suggestions": suggestions.len(),
            "categories": {
                "clippy": suggestions.iter().filter(|s| s.get("type") == Some(&json!("clippy"))).count(),
                "general": suggestions.iter().filter(|s| s.get("type") == Some(&json!("general"))).count(),
                "formatting": suggestions.iter().filter(|s| s.get("type") == Some(&json!("formatting"))).count()
            }
        }))
    }
}