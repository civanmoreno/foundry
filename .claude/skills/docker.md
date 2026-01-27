# Docker Module

## Location
[src/docker/mod.rs](src/docker/mod.rs)

## Purpose
Handles all Docker interactions using Bollard (native Docker API client for Rust).

## Docker Labels

Foundry uses labels to track managed resources (no internal state/daemon):

```
foundry.project = my-app      # Project identifier
foundry.service = app         # Service name
foundry.env     = local|test  # Environment (local or test)
foundry.managed = true        # Marks resource as Foundry-managed
```

## Key Structures

```rust
pub struct DockerClient {
    client: Docker,  // Bollard Docker client
}

pub struct Labels;  // Helper for building label sets
```

## API

```rust
// Create client (verifies Docker is available)
let docker = DockerClient::new().await?;

// Check Docker availability
let available = docker.is_available().await;

// Operations
docker.up(&config, Some("service_name")).await?;
docker.down(&config, None).await?;
docker.clean(&config, remove_volumes).await?;
docker.status(&config).await?;
```

## Labels Helper

```rust
// Build labels for a service
let labels = Labels::for_service("my-app", "web", "local");
// Returns: HashMap with foundry.project, foundry.service, foundry.env, foundry.managed
```

## Bollard Patterns

```rust
use bollard::Docker;
use bollard::container::{CreateContainerOptions, Config as ContainerConfig};
use bollard::network::CreateNetworkOptions;

// Connect to Docker
let docker = Docker::connect_with_local_defaults()?;

// Create container
let options = CreateContainerOptions { name: "my-container", .. };
let config = ContainerConfig { image: Some("nginx"), .. };
docker.create_container(Some(options), config).await?;

// Start container
docker.start_container("my-container", None).await?;

// List containers with label filter
let filters = HashMap::from([("label", vec!["foundry.managed=true"])]);
let options = ListContainersOptions { filters, .. };
let containers = docker.list_containers(Some(options)).await?;
```

## Guidelines
- Always use labels to track resources
- Check Docker availability before operations
- Use async/await for all Docker operations
- Handle errors gracefully with meaningful messages
- Clean up resources in reverse order of creation
