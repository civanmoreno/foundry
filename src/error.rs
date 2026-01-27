use thiserror::Error;

#[derive(Error, Debug)]
pub enum FoundryError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Configuration file not found: {0}")]
    ConfigNotFound(String),

    #[error("Docker error: {0}")]
    Docker(#[from] bollard::errors::Error),

    #[error("Docker operation failed: {0}")]
    DockerError(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Project not initialized. Run 'foundry init' or create a foundry.yaml file")]
    ProjectNotInitialized,

    #[error("Docker is not running or not accessible")]
    DockerNotAvailable,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, FoundryError>;
