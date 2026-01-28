use bollard::Docker;
use bollard::container::{
    CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::models::{EndpointSettings, HostConfig};
use bollard::network::CreateNetworkOptions;
use futures::StreamExt;
use std::collections::HashMap;
use std::default::Default;
use std::io::Write;

use crate::config::{Config, Service};
use crate::error::{FoundryError, Result};

/// Docker label prefix for Foundry-managed resources
pub const LABEL_PREFIX: &str = "foundry";

/// Standard labels applied to all Foundry-managed resources
pub struct Labels;

impl Labels {
    pub fn project(name: &str) -> (String, String) {
        (format!("{}.project", LABEL_PREFIX), name.to_string())
    }

    pub fn service(name: &str) -> (String, String) {
        (format!("{}.service", LABEL_PREFIX), name.to_string())
    }

    pub fn env(env: &str) -> (String, String) {
        (format!("{}.env", LABEL_PREFIX), env.to_string())
    }

    pub fn managed() -> (String, String) {
        (format!("{}.managed", LABEL_PREFIX), "true".to_string())
    }

    /// Build labels HashMap for a service
    pub fn for_service(project: &str, service: &str, env: &str) -> HashMap<String, String> {
        let mut labels = HashMap::new();
        let (k, v) = Self::project(project);
        labels.insert(k, v);
        let (k, v) = Self::service(service);
        labels.insert(k, v);
        let (k, v) = Self::env(env);
        labels.insert(k, v);
        let (k, v) = Self::managed();
        labels.insert(k, v);
        labels
    }
}

/// Docker client wrapper for Foundry operations
pub struct DockerClient {
    client: Docker,
}

impl DockerClient {
    /// Create a new Docker client
    pub async fn new() -> Result<Self> {
        let client = Docker::connect_with_local_defaults()
            .map_err(|_| FoundryError::DockerNotAvailable)?;

        // Verify connection
        client
            .ping()
            .await
            .map_err(|_| FoundryError::DockerNotAvailable)?;

        Ok(Self { client })
    }

    /// Get the underlying Bollard client
    #[allow(dead_code)]
    pub fn inner(&self) -> &Docker {
        &self.client
    }

    /// Check if Docker is available
    #[allow(dead_code)]
    pub async fn is_available(&self) -> bool {
        self.client.ping().await.is_ok()
    }

    /// Get or create network for a project
    async fn ensure_network(&self, project: &str) -> Result<String> {
        let network_name = format!("{}-network", project);

        // Check if network exists
        match self.client.inspect_network::<&str>(&network_name, None).await {
            Ok(_) => return Ok(network_name),
            Err(_) => {}
        }

        // Create the network
        let config = CreateNetworkOptions {
            name: network_name.clone(),
            driver: "bridge".to_string(),
            labels: Labels::for_service(project, "network", "development"),
            ..Default::default()
        };

        self.client.create_network(config).await.map_err(|e| {
            FoundryError::DockerError(format!("Failed to create network: {}", e))
        })?;

        Ok(network_name)
    }

    /// Generate nginx config for PHP-FPM integration
    fn generate_nginx_config(&self, project: &str, nginx_root: &str, php_root: &str) -> String {
        let php_container = format!("{}-php", project);
        format!(
            r#"resolver 127.0.0.11 valid=30s;

server {{
    listen 80;
    server_name localhost;
    root {nginx_root};
    index index.php index.html index.htm;

    location / {{
        try_files $uri $uri/ /index.php?$query_string;
    }}

    location ~ \.php$ {{
        set $upstream {php_container}:9000;
        fastcgi_pass $upstream;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME {php_root}$fastcgi_script_name;
        include fastcgi_params;
    }}

    location ~ /\.ht {{
        deny all;
    }}
}}
"#,
            nginx_root = nginx_root,
            php_root = php_root,
            php_container = php_container
        )
    }

    /// Write nginx config to temp file and return path
    fn write_nginx_config(&self, project: &str, nginx_root: &str, php_root: &str) -> Result<std::path::PathBuf> {
        let config_content = self.generate_nginx_config(project, nginx_root, php_root);
        let config_dir = std::env::temp_dir().join("foundry").join(project);
        std::fs::create_dir_all(&config_dir)?;

        // Use project name for the config file
        let config_path = config_dir.join(format!("{}.conf", project));
        let mut file = std::fs::File::create(&config_path)?;
        file.write_all(config_content.as_bytes())?;

        Ok(config_path)
    }

    /// Start all services defined in the configuration
    pub async fn up(&self, config: &Config, service_filter: Option<&str>) -> Result<()> {
        tracing::info!("Starting services for project '{}'", config.name);

        // Get normalized services
        let all_services = config.get_services()?;

        // Filter services if requested
        let services_to_start: Vec<&Service> = if let Some(filter) = service_filter {
            all_services.iter().filter(|s| s.name == filter).collect()
        } else {
            all_services.iter().collect()
        };

        if services_to_start.is_empty() {
            return Err(FoundryError::ConfigNotFound(format!(
                "No services found{}",
                service_filter.map(|f| format!(" matching '{}'", f)).unwrap_or_default()
            )));
        }

        // Check if we have both nginx and php (for auto-configuration)
        let has_php = all_services.iter().any(|s| s.name == "php");
        let has_nginx = all_services.iter().any(|s| s.name == "nginx");
        let needs_network = has_php && has_nginx;

        // Create network if needed for inter-container communication
        let network_name = if needs_network {
            Some(self.ensure_network(&config.name).await?)
        } else {
            None
        };

        // Generate nginx config if both nginx and php are present
        let nginx_config_path = if has_php && has_nginx {
            // Get the root paths for both services
            let nginx_root = all_services
                .iter()
                .find(|s| s.name == "nginx")
                .and_then(|s| s.root.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("/usr/share/nginx/html");
            let php_root = all_services
                .iter()
                .find(|s| s.name == "php")
                .and_then(|s| s.root.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("/var/www/html");
            Some(self.write_nginx_config(&config.name, nginx_root, php_root)?)
        } else {
            None
        };

        // Sort services: nginx should start last (after php-fpm is ready)
        let mut services_to_start = services_to_start;
        services_to_start.sort_by(|a, b| {
            if a.name == "nginx" {
                std::cmp::Ordering::Greater
            } else if b.name == "nginx" {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });

        // Create and start containers
        for service in services_to_start {
            let container_name = format!("{}-{}", config.name, service.name);

            // Check if container already exists
            if let Some(state) = self.container_state(&container_name).await {
                if state == "running" {
                    println!("✓ {} already running", service.name);
                    continue;
                } else {
                    // Container exists but stopped, start it
                    println!("Starting {}...", service.name);
                    self.client
                        .start_container(&container_name, None::<StartContainerOptions<String>>)
                        .await?;

                    let port_info = service
                        .port
                        .map(|p| format!(" (port {})", p))
                        .unwrap_or_default();
                    println!("✓ {} started{}", service.name, port_info);
                    continue;
                }
            }

            // Container doesn't exist, create it
            println!("Starting {}...", service.name);

            // Pull image if not exists
            if !self.image_exists(&service.image).await {
                self.pull_image(&service.image).await?;
            }

            let labels = Labels::for_service(&config.name, &service.name, "development");

            // Build port bindings
            let (port_bindings, exposed_ports) = self.build_port_config(service);

            // Build volume bindings if service has a root path (container destination)
            let binds = if let Some(ref container_path) = service.root {
                // Get current working directory
                let cwd = std::env::current_dir().unwrap_or_default();

                // Use config.root as source, or current directory if not specified
                let host_path = match &config.root {
                    Some(root) => {
                        // Resolve relative path to absolute
                        let path = std::path::Path::new(root);
                        if path.is_absolute() {
                            root.clone()
                        } else {
                            cwd.join(path).to_string_lossy().to_string()
                        }
                    }
                    None => cwd.to_string_lossy().to_string(),
                };

                // Validate that root is a directory
                let host_path_obj = std::path::Path::new(&host_path);
                if !host_path_obj.exists() {
                    return Err(FoundryError::ConfigNotFound(format!(
                        "root path '{}' does not exist",
                        host_path
                    )));
                }
                if !host_path_obj.is_dir() {
                    return Err(FoundryError::ConfigNotFound(format!(
                        "root path '{}' must be a directory, not a file",
                        host_path
                    )));
                }

                Some(vec![format!("{}:{}", host_path, container_path)])
            } else {
                None
            };

            // Add nginx config mount if this is nginx and we have the config
            // Mount as default.conf to override nginx's built-in config
            let mut final_binds = binds.unwrap_or_default();
            if service.name == "nginx" {
                if let Some(ref config_path) = nginx_config_path {
                    final_binds.push(format!(
                        "{}:/etc/nginx/conf.d/default.conf:ro",
                        config_path.display()
                    ));
                }
            }

            // Build host config with port bindings, volumes, and network
            let host_config = HostConfig {
                port_bindings: Some(port_bindings),
                binds: if final_binds.is_empty() {
                    None
                } else {
                    Some(final_binds)
                },
                ..Default::default()
            };

            // Build environment variables
            let env_vars: Vec<String> = service
                .env
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();

            let config_opts = CreateContainerOptions {
                name: container_name.clone(),
                platform: None,
            };

            // Build network config if needed
            let networking_config = network_name.as_ref().map(|net| {
                let mut endpoints = HashMap::new();
                endpoints.insert(
                    net.clone(),
                    EndpointSettings {
                        ..Default::default()
                    },
                );
                bollard::container::NetworkingConfig {
                    endpoints_config: endpoints,
                }
            });

            // Build container spec
            let container_spec = bollard::container::Config {
                image: Some(service.image.clone()),
                labels: Some(labels),
                exposed_ports: Some(exposed_ports),
                host_config: Some(host_config),
                networking_config,
                env: if env_vars.is_empty() {
                    None
                } else {
                    Some(env_vars)
                },
                ..Default::default()
            };

            // Create the container
            let create_response = self
                .client
                .create_container(Some(config_opts), container_spec)
                .await;

            match create_response {
                Ok(response) => {
                    // Start the container
                    self.client
                        .start_container(&response.id, None::<StartContainerOptions<String>>)
                        .await?;

                    let port_info = service
                        .port
                        .map(|p| format!(" (port {})", p))
                        .unwrap_or_default();
                    println!("✓ {} started{}", service.name, port_info);
                }
                Err(e) => {
                    return Err(FoundryError::DockerError(format!(
                        "Failed to create container {}: {}",
                        service.name, e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Build port configuration for a service
    fn build_port_config(
        &self,
        service: &Service,
    ) -> (
        HashMap<String, Option<Vec<bollard::models::PortBinding>>>,
        HashMap<String, HashMap<(), ()>>,
    ) {
        let mut port_bindings = HashMap::new();
        let mut exposed_ports = HashMap::new();

        if let Some(port) = service.port {
            let container_port = format!("{}/tcp", port);
            port_bindings.insert(
                container_port.clone(),
                Some(vec![bollard::models::PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(port.to_string()),
                }]),
            );
            exposed_ports.insert(container_port, HashMap::new());
        }

        (port_bindings, exposed_ports)
    }

    /// Pull a Docker image
    async fn pull_image(&self, image: &str) -> Result<()> {
        println!("  Pulling {}...", image);

        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        let mut stream = self.client.create_image(Some(options), None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(_) => {}
                Err(e) => {
                    return Err(FoundryError::DockerError(format!(
                        "Failed to pull image {}: {}",
                        image, e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check if image exists locally
    async fn image_exists(&self, image: &str) -> bool {
        self.client.inspect_image(image).await.is_ok()
    }

    /// Check if container exists and return its state
    async fn container_state(&self, name: &str) -> Option<String> {
        match self.client.inspect_container(name, None).await {
            Ok(info) => info.state.and_then(|s| s.status.map(|st| st.to_string())),
            Err(_) => None,
        }
    }


    /// Stop all running services
    pub async fn down(&self, config: &Config, service_filter: Option<&str>) -> Result<()> {
        let containers = self.list_project_containers(&config.name).await?;

        if containers.is_empty() {
            println!("No running services found");
            return Ok(());
        }

        for container in containers {
            let name = container
                .names
                .as_ref()
                .and_then(|n| n.first())
                .map(|n| n.trim_start_matches('/'))
                .unwrap_or("unknown");

            // Filter by service if specified
            if let Some(filter) = service_filter {
                let service_name = name.strip_prefix(&format!("{}-", config.name)).unwrap_or(name);
                if service_name != filter {
                    continue;
                }
            }

            if let Some(id) = &container.id {
                println!("Stopping {}...", name);
                self.client
                    .stop_container(id, Some(StopContainerOptions { t: 10 }))
                    .await
                    .ok(); // Ignore error if already stopped
                println!("✓ {} stopped", name);
            }
        }

        Ok(())
    }

    /// Remove all containers, networks, and optionally volumes
    pub async fn clean(&self, config: &Config, _remove_volumes: bool) -> Result<()> {
        let containers = self.list_project_containers(&config.name).await?;

        if containers.is_empty() {
            println!("No containers to clean");
            return Ok(());
        }

        for container in containers {
            let name = container
                .names
                .as_ref()
                .and_then(|n| n.first())
                .map(|n| n.trim_start_matches('/'))
                .unwrap_or("unknown");

            if let Some(id) = &container.id {
                println!("Removing {}...", name);
                self.client
                    .remove_container(
                        id,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await?;
                println!("✓ {} removed", name);
            }
        }

        Ok(())
    }

    /// List all containers for a project
    async fn list_project_containers(
        &self,
        project: &str,
    ) -> Result<Vec<bollard::models::ContainerSummary>> {
        let mut filters = HashMap::new();
        filters.insert(
            "label".to_string(),
            vec![format!("foundry.project={}", project)],
        );

        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        let containers = self.client.list_containers(Some(options)).await?;
        Ok(containers)
    }

    /// Show status of all services
    pub async fn status(&self, config: &Config) -> Result<()> {
        let containers = self.list_project_containers(&config.name).await?;

        if containers.is_empty() {
            println!("No services running for project '{}'", config.name);
            return Ok(());
        }

        println!("\nProject: {}\n", config.name);
        println!("{:<20} {:<25} {:<15}", "SERVICE", "IMAGE", "STATUS");
        println!("{}", "-".repeat(60));

        for container in containers {
            let name = container
                .names
                .as_ref()
                .and_then(|n| n.first())
                .map(|n| n.trim_start_matches('/'))
                .map(|n| n.strip_prefix(&format!("{}-", config.name)).unwrap_or(n))
                .unwrap_or("unknown");

            let image = container.image.as_deref().unwrap_or("unknown");
            let state = container.state.as_deref().unwrap_or("unknown");

            println!("{:<20} {:<25} {:<15}", name, image, state);
        }

        Ok(())
    }
}
