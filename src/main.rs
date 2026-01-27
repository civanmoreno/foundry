mod cli;
mod config;
mod docker;
mod error;

use cli::{Cli, Commands};
use config::Config;
use docker::DockerClient;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse_args();

    // Set log level based on verbose flag
    if cli.verbose {
        tracing::debug!("Verbose mode enabled");
    }

    match cli.command {
        Commands::Up { service, detach: _ } => {
            let config = Config::load()?;
            let docker = DockerClient::new().await?;
            docker.up(&config, service.as_deref()).await?;
            println!("Services started successfully");
        }

        Commands::Down { service } => {
            let config = Config::load()?;
            let docker = DockerClient::new().await?;
            docker.down(&config, service.as_deref()).await?;
            println!("Services stopped");
        }

        Commands::Clean { volumes } => {
            let config = Config::load()?;
            let docker = DockerClient::new().await?;
            docker.clean(&config, volumes).await?;
            println!("Cleanup complete");
        }

        Commands::Test { args } => {
            let _config = Config::load()?;
            let _docker = DockerClient::new().await?;
            // TODO: Implement test environment
            println!("Running tests with args: {:?}", args);
        }

        Commands::Exec { service, command } => {
            let _config = Config::load()?;
            let _docker = DockerClient::new().await?;
            // TODO: Implement exec
            println!("Executing in {}: {:?}", service, command);
        }

        Commands::Status => {
            let config = Config::load()?;
            let docker = DockerClient::new().await?;
            docker.status(&config).await?;
        }

        Commands::Logs {
            service,
            follow: _,
            tail: _,
        } => {
            let _config = Config::load()?;
            let _docker = DockerClient::new().await?;
            // TODO: Implement logs
            println!("Showing logs for: {:?}", service.unwrap_or("all".into()));
        }

        Commands::Init { name } => {
            let project_name = name.unwrap_or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                    .unwrap_or_else(|| "my-project".to_string())
            });

            let config = Config::default_config(&project_name);
            config.save_to("foundry.yaml")?;
            println!("Created foundry.yaml for project: {}", project_name);
        }
    }

    Ok(())
}
