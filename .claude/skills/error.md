# Error Handling Module

## Location
[src/error.rs](src/error.rs)

## Purpose
Defines custom error types for Foundry using thiserror.

## Error Types

```rust
pub enum FoundryError {
    Config(String),           // Configuration errors
    ConfigNotFound(String),   // Missing config file
    Docker(bollard::Error),   // Docker API errors
    ServiceNotFound(String),  // Unknown service name
    ProjectNotInitialized,    // No foundry.yaml found
    DockerNotAvailable,       // Docker not running
    Io(std::io::Error),       // File system errors
    Yaml(serde_yaml::Error),  // YAML parsing errors
}

pub type Result<T> = std::result::Result<T, FoundryError>;
```

## Usage

```rust
use crate::error::{FoundryError, Result};

fn load_config() -> Result<Config> {
    if !path.exists() {
        return Err(FoundryError::ConfigNotFound(path.to_string()));
    }
    // ...
}

// Auto-conversion from other error types
fn read_file() -> Result<String> {
    let content = std::fs::read_to_string(path)?;  // io::Error -> FoundryError::Io
    Ok(content)
}
```

## Guidelines
- Use thiserror for derive macros
- Provide clear, actionable error messages
- Use `#[from]` for automatic error conversion
- Keep errors specific to the domain
- Consider user experience in error messages

## Adding New Errors

```rust
#[derive(Error, Debug)]
pub enum FoundryError {
    // Add new variant
    #[error("Container failed to start: {0}")]
    ContainerStartFailed(String),
}
```
