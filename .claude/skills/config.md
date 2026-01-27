# Configuration Module

## Location
[src/config/mod.rs](src/config/mod.rs)

## Purpose
Handles loading, parsing, and saving the `foundry.yaml` configuration file.

## Configuration Schema

```yaml
name: my-app                    # Project name (required)

services:                       # Service definitions
  app:
    image: node:20              # Docker image (required)
    command: npm run dev        # Override default command
    workdir: /app               # Working directory
    ports:
      - "3000:3000"             # Port mappings (host:container)
    env:
      NODE_ENV: development     # Environment variables
    volumes:
      - "./src:/app/src"        # Volume mounts
    depends_on:
      - db                      # Service dependencies
```

## Key Structures

```rust
pub struct Config {
    pub name: String,
    pub services: HashMap<String, Service>,
}

pub struct Service {
    pub image: String,
    pub command: Option<String>,
    pub workdir: Option<String>,
    pub ports: Vec<String>,
    pub env: HashMap<String, String>,
    pub volumes: Vec<String>,
    pub depends_on: Vec<String>,
}
```

## API

```rust
// Load from current directory
let config = Config::load()?;

// Load from specific path
let config = Config::load_from("path/to/foundry.yaml")?;

// Create default config
let config = Config::default_config("my-project");

// Save to file
config.save_to("foundry.yaml")?;
```

## Guidelines
- Use Serde for serialization/deserialization
- All optional fields should have sensible defaults
- Validate configuration before use
- Keep close to Docker concepts (no magic behavior)
