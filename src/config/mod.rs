use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{FoundryError, Result};

/// Main configuration structure representing foundry.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project name (used for labeling Docker resources)
    pub name: String,

    /// Services defined in the project
    #[serde(default)]
    pub services: HashMap<String, Service>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Docker image to use
    pub image: String,

    /// Command to run (overrides image default)
    #[serde(default)]
    pub command: Option<String>,

    /// Working directory inside container
    #[serde(default)]
    pub workdir: Option<String>,

    /// Port mappings (host:container)
    #[serde(default)]
    pub ports: Vec<String>,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Volume mounts
    #[serde(default)]
    pub volumes: Vec<String>,

    /// Services this service depends on
    #[serde(default)]
    pub depends_on: Vec<String>,
}

impl Config {
    /// Load configuration from foundry.yaml in the current directory
    pub fn load() -> Result<Self> {
        Self::load_from("foundry.yaml")
    }

    /// Load configuration from a specific path
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(FoundryError::ConfigNotFound(
                path.display().to_string(),
            ));
        }

        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;

        Ok(config)
    }

    /// Create a default configuration
    pub fn default_config(name: &str) -> Self {
        Config {
            name: name.to_string(),
            services: HashMap::new(),
        }
    }

    /// Save configuration to a file
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
