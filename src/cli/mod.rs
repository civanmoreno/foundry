use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "foundry")]
#[command(author, version, about = "Fast, simple, and predictable local development environments")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start all services defined in foundry.yaml
    Up {
        /// Specific service to start (starts all if not specified)
        service: Option<String>,

        /// Run in detached mode
        #[arg(short, long)]
        detach: bool,
    },

    /// Stop all running services
    Down {
        /// Specific service to stop (stops all if not specified)
        service: Option<String>,
    },

    /// Remove all containers, networks, and volumes
    Clean {
        /// Also remove volumes (data will be lost)
        #[arg(long)]
        volumes: bool,
    },

    /// Run tests in an isolated environment
    Test {
        /// Command to run (defaults to project test command)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Execute a command inside a running service
    Exec {
        /// Service name
        service: String,

        /// Command to execute
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },

    /// Show status of all services
    Status,

    /// Show logs from services
    Logs {
        /// Specific service to show logs for
        service: Option<String>,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show
        #[arg(short, long, default_value = "100")]
        tail: u32,
    },

    /// Initialize a new foundry.yaml file
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}
