use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonInfo {
    pub port: u16,
    pub project_path: String,
    pub pid: Option<u32>,
    pub started_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonState {
    daemons: HashMap<String, DaemonInfo>,
}

impl DaemonState {
    pub fn new() -> Self {
        Self {
            daemons: HashMap::new(),
        }
    }

    /// Get the state file path in user's home directory
    fn get_state_file() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Could not determine home directory")?;
            
        Ok(home.join(".mcp-rust-analyzer-state.json"))
    }

    /// Load state from file, or create new if doesn't exist
    pub fn load() -> Result<Self> {
        let state_file = Self::get_state_file()?;
        
        if !state_file.exists() {
            debug!("State file does not exist, creating new state");
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&state_file)
            .context("Failed to read state file")?;
            
        let state: DaemonState = serde_json::from_str(&content)
            .context("Failed to parse state file")?;
            
        debug!("Loaded state with {} daemons", state.daemons.len());
        Ok(state)
    }

    /// Save state to file
    pub fn save(&self) -> Result<()> {
        let state_file = Self::get_state_file()?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = state_file.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create state directory")?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize state")?;
            
        fs::write(&state_file, content)
            .context("Failed to write state file")?;
            
        debug!("Saved state to {}", state_file.display());
        Ok(())
    }

    /// Normalize project path to absolute canonical path
    fn normalize_path(path: &str) -> Result<String> {
        let path = Path::new(path);
        let canonical = if path.is_absolute() {
            path.canonicalize()?
        } else {
            std::env::current_dir()?.join(path).canonicalize()?
        };
        Ok(canonical.to_string_lossy().to_string())
    }

    /// Find an available port
    pub fn find_available_port() -> Result<u16> {
        // Try ports starting from 3000
        for port in 3000..=9999 {
            if Self::is_port_available(port) {
                return Ok(port);
            }
        }
        anyhow::bail!("No available ports found in range 3000-9999")
    }

    /// Check if a port is available
    fn is_port_available(port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    /// Register a new daemon
    pub fn register_daemon(&mut self, project_path: &str, port: u16, pid: Option<u32>) -> Result<()> {
        let normalized_path = Self::normalize_path(project_path)?;
        
        let daemon_info = DaemonInfo {
            port,
            project_path: normalized_path.clone(),
            pid,
            started_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        };

        self.daemons.insert(normalized_path, daemon_info);
        self.save()?;
        
        info!("Registered daemon for {} on port {}", project_path, port);
        Ok(())
    }

    /// Find daemon for current directory
    pub fn find_daemon_for_current_dir() -> Result<Option<DaemonInfo>> {
        let current_dir = std::env::current_dir()?;
        let normalized_path = current_dir.canonicalize()?.to_string_lossy().to_string();
        
        debug!("Looking for daemon in current dir: {}", normalized_path);
        
        let state = Self::load()?;
        
        debug!("Loaded state with {} registered daemons", state.daemons.len());
        
        if let Some(daemon_info) = state.daemons.get(&normalized_path) {
            // Verify the daemon is still running
            if Self::is_daemon_running(daemon_info.port) {
                debug!("Found running daemon for {} on port {}", normalized_path, daemon_info.port);
                return Ok(Some(daemon_info.clone()));
            } else {
                warn!("Daemon registered for {} on port {} is not responding", normalized_path, daemon_info.port);
                // Clean up dead daemon
                let mut state = state;
                state.daemons.remove(&normalized_path);
                state.save()?;
                return Ok(None);
            }
        }
        
        debug!("No daemon found for {}", normalized_path);
        Ok(None)
    }

    /// Check if daemon is running by trying to connect
    fn is_daemon_running(port: u16) -> bool {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap();
            
        match client.get(&format!("http://localhost:{}/", port)).send() {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Remove daemon from state
    pub fn unregister_daemon(&mut self, project_path: &str) -> Result<Option<DaemonInfo>> {
        let normalized_path = Self::normalize_path(project_path)?;
        
        if let Some(daemon_info) = self.daemons.remove(&normalized_path) {
            self.save()?;
            info!("Unregistered daemon for {} on port {}", project_path, daemon_info.port);
            Ok(Some(daemon_info))
        } else {
            Ok(None)
        }
    }

    /// Remove daemon for current directory
    pub fn unregister_daemon_for_current_dir() -> Result<Option<DaemonInfo>> {
        let current_dir = std::env::current_dir()?;
        let normalized_path = current_dir.canonicalize()?.to_string_lossy().to_string();
        
        let mut state = Self::load()?;
        
        if let Some(daemon_info) = state.daemons.remove(&normalized_path) {
            state.save()?;
            info!("Unregistered daemon for {} on port {}", normalized_path, daemon_info.port);
            Ok(Some(daemon_info))
        } else {
            Ok(None)
        }
    }

    /// Get all registered daemons
    pub fn get_all_daemons(&self) -> &HashMap<String, DaemonInfo> {
        &self.daemons
    }

    /// Clean up dead daemons from state
    pub fn cleanup_dead_daemons(&mut self) -> Result<()> {
        let mut dead_daemons = Vec::new();
        
        for (path, daemon_info) in &self.daemons {
            if !Self::is_daemon_running(daemon_info.port) {
                dead_daemons.push(path.clone());
            }
        }
        
        for path in dead_daemons {
            info!("Cleaning up dead daemon for {}", path);
            self.daemons.remove(&path);
        }
        
        if !self.daemons.is_empty() {
            self.save()?;
        }
        
        Ok(())
    }
}