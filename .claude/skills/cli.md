# CLI Module

## Location
[src/cli/mod.rs](src/cli/mod.rs)

## Purpose
Handles command-line interface parsing and argument handling using Clap.

## Available Commands

| Command | Description | Arguments |
|---------|-------------|-----------|
| `up` | Start services | `[service]`, `--detach` |
| `down` | Stop services | `[service]` |
| `clean` | Remove resources | `--volumes` |
| `test` | Run tests in isolated env | `[args...]` |
| `exec` | Execute command in service | `<service>`, `<command...>` |
| `status` | Show service status | - |
| `logs` | Show service logs | `[service]`, `--follow`, `--tail` |
| `init` | Initialize foundry.yaml | `--name` |

## Global Flags
- `-v, --verbose` - Enable verbose output

## Key Structures

```rust
pub struct Cli {
    pub command: Commands,
    pub verbose: bool,
}

pub enum Commands {
    Up { service: Option<String>, detach: bool },
    Down { service: Option<String> },
    Clean { volumes: bool },
    Test { args: Vec<String> },
    Exec { service: String, command: Vec<String> },
    Status,
    Logs { service: Option<String>, follow: bool, tail: u32 },
    Init { name: Option<String> },
}
```

## Usage Pattern

```rust
let cli = Cli::parse_args();
match cli.command {
    Commands::Up { service, detach } => { /* handle */ }
    // ...
}
```

## Guidelines
- Use Clap derive macros for argument parsing
- Keep command descriptions clear and concise
- Use `trailing_var_arg = true` for commands that accept variable arguments
- Global flags should use `global = true`
