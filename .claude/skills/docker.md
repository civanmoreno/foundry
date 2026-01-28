# Docker Module

## Location
[src/docker/mod.rs](src/docker/mod.rs)

## Purpose
Handles all Docker interactions using Bollard (native Docker API client for Rust).

## Commands

| Command | Status | Description |
|---------|--------|-------------|
| `up` | ✅ | Pull images, create/restart containers |
| `down` | ✅ | Stop running containers |
| `clean` | ✅ | Remove containers |
| `status` | ✅ | List containers and their state |

## Behavior

### `foundry up`
1. Si contenedor existe y está **running** → skip
2. Si contenedor existe y está **stopped** → restart
3. Si contenedor no existe → pull image + create + start

**Auto-configuración nginx + PHP:**
- Si nginx y php están en el config, se auto-configura:
  - Crea red Docker para comunicación inter-container
  - Genera nginx.conf que pasa .php a php-fpm
  - PHP se inicia antes que nginx (orden de dependencias)
  - El config se monta en `/etc/nginx/conf.d/default.conf`

### `foundry down`
- Detiene contenedores (no los elimina)
- Contenedores quedan en estado "exited"

### `foundry clean`
- Elimina contenedores (force)
- Después de clean, `up` crea nuevos contenedores

## Docker Labels

```
foundry.project = my-app
foundry.service = mysql
foundry.env     = development
foundry.managed = true
```

## Container Naming

```
{project_name}-{service_name}
```
Example: `my-app-mysql`, `my-app-redis`

## Key Functions

```rust
// Check container state
async fn container_state(&self, name: &str) -> Option<String>

// Check if image exists locally
async fn image_exists(&self, image: &str) -> bool

// Pull image from registry
async fn pull_image(&self, image: &str) -> Result<()>

// List containers by project label
async fn list_project_containers(&self, project: &str) -> Result<Vec<ContainerSummary>>

// Create Docker network for project
async fn ensure_network(&self, project: &str) -> Result<String>

// Generate nginx config for PHP-FPM
fn generate_nginx_config(&self, project: &str, nginx_root: &str, php_root: &str) -> String

// Write nginx config to temp file
fn write_nginx_config(&self, project: &str, nginx_root: &str, php_root: &str) -> Result<PathBuf>
```

## Bollard Imports

```rust
use bollard::Docker;
use bollard::container::{
    CreateContainerOptions, ListContainersOptions,
    RemoveContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::models::HostConfig;
use futures::StreamExt;
```
