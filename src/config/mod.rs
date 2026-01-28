use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{FoundryError, Result};

/// Service version specification (short format like "20" or long format with details)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServiceSpec {
    // Short format: just a version string or number
    ShortNum(u32),
    ShortFloat(f64),
    ShortStr(String),
    // Long format: with additional configuration
    Long(ServiceConfig),
}

/// Service configuration (long format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub version: Option<serde_yaml::Value>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub root: Option<String>,
}

/// Main configuration structure representing foundry.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project name (used for labeling Docker resources)
    pub name: String,

    /// Services defined in the project (supports both short and long format)
    #[serde(default, flatten)]
    pub services: HashMap<String, ServiceSpec>,
}

/// Normalized service for Docker operations
#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub image: String,
    #[allow(dead_code)]
    pub version: String,
    pub port: Option<u16>,
    pub env: HashMap<String, String>,
    pub root: Option<String>,
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

    /// Get normalized services from the configuration
    pub fn get_services(&self) -> Result<Vec<Service>> {
        let mut services = Vec::new();

        for (name, spec) in &self.services {
            // Skip "name" as it's a configuration key, not a service
            if name == "name" {
                continue;
            }

            let (version, port, custom_image, mut env, custom_root) = match spec {
                ServiceSpec::ShortNum(v) => (v.to_string(), None, None, HashMap::new(), None),
                ServiceSpec::ShortFloat(v) => (v.to_string(), None, None, HashMap::new(), None),
                ServiceSpec::ShortStr(v) => (v.clone(), None, None, HashMap::new(), None),
                ServiceSpec::Long(config) => {
                    let ver = config
                        .version
                        .as_ref()
                        .map(|v| match v {
                            serde_yaml::Value::Number(n) => n.to_string(),
                            serde_yaml::Value::String(s) => s.clone(),
                            _ => "latest".to_string(),
                        })
                        .unwrap_or_else(|| "latest".to_string());
                    (
                        ver,
                        config.port,
                        config.image.clone(),
                        config.env.clone(),
                        config.root.clone(),
                    )
                }
            };

            // Add default env vars for known services
            match name.as_str() {
                "mysql" => {
                    env.entry("MYSQL_ROOT_PASSWORD".to_string())
                        .or_insert_with(|| "secret".to_string());
                    env.entry("MYSQL_DATABASE".to_string())
                        .or_insert_with(|| "app".to_string());
                }
                "postgres" => {
                    env.entry("POSTGRES_PASSWORD".to_string())
                        .or_insert_with(|| "secret".to_string());
                    env.entry("POSTGRES_DB".to_string())
                        .or_insert_with(|| "app".to_string());
                }
                "mongodb" => {
                    env.entry("MONGO_INITDB_ROOT_USERNAME".to_string())
                        .or_insert_with(|| "root".to_string());
                    env.entry("MONGO_INITDB_ROOT_PASSWORD".to_string())
                        .or_insert_with(|| "secret".to_string());
                }
                _ => {}
            }

            // Map service names to default Docker images (official images)
            let image = custom_image.unwrap_or_else(|| match name.as_str() {
                "node" => format!("node:{}-alpine", version),
                "redis" => format!("redis:{}-alpine", version),
                "php" => format!("php:{}-fpm-alpine", version),
                "mysql" => format!("mysql:{}", version),
                "postgres" => format!("postgres:{}-alpine", version),
                "mongodb" => format!("mongo:{}", version),
                "nginx" => format!("nginx:{}-alpine", version),
                "python" => format!("python:{}-slim", version),
                "ruby" => format!("ruby:{}-slim", version),
                "golang" => format!("golang:{}-alpine", version),
                _ => format!("{}:{}", name, version),
            });

            // Default root paths for services that need project files
            let root = custom_root.or_else(|| match name.as_str() {
                "php" => Some("/var/www/html".to_string()),
                "nginx" => Some("/usr/share/nginx/html".to_string()),
                "node" => Some("/app".to_string()),
                "python" => Some("/app".to_string()),
                "ruby" => Some("/app".to_string()),
                "golang" => Some("/app".to_string()),
                _ => None,
            });

            services.push(Service {
                name: name.clone(),
                image,
                version,
                port,
                env,
                root,
            });
        }

        Ok(services)
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
