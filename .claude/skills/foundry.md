# Foundry - Local Development Environment Tool

## About This Project

Foundry es una herramienta de desarrollo local escrita en Rust que permite a cualquier desarrollador levantar su entorno de trabajo con un solo comando.

Funciona como una alternativa moderna a Lando Dev, usando directamente la API nativa de Docker (Bollard) para ofrecer arranques más rápidos, configuración simple y entornos predecibles.

Foundry elimina la complejidad de Docker Compose y DevOps, enfocándose en una excelente experiencia de desarrollo, onboarding rápido y entornos de prueba efímeros y confiables.

## Core Promise

> Any developer should be able to clone a repository and run the project locally with one command.

## Technical Stack

- **Language:** Rust
- **Async Runtime:** Tokio
- **Docker API:** Bollard (native Docker API client for Rust)
- **Execution Model:** Single static binary
- **State Management:** Docker labels (no daemon, no database)

## Core Concepts

```
Project → Services → Resources → Lifecycle
```

- **Project:** Represents a repository, lives in a directory, unit of isolation
- **Service:** One service = one container, explicit configuration
- **Resources:** Containers, Networks, Volumes (managed via Docker labels)
- **Lifecycle:** `up`, `down`, `clean`, `test`

## Docker Labels for State

Foundry uses Docker labels instead of internal state:

```
foundry.project = my-app
foundry.service = app
foundry.env     = local | test
foundry.managed = true
```

## CLI Commands

- `foundry up` - Create and start resources
- `foundry down` - Stop resources
- `foundry clean` - Remove resources
- `foundry test` - Run in ephemeral isolated environment
- `foundry exec <service> <command>` - Execute commands inside containers

## Development Guidelines

When working on this project:

1. Follow Rust best practices and idioms
2. Use async/await with Tokio for all I/O operations
3. Use Bollard crate for Docker API interactions
4. Keep the CLI simple and predictable
5. Prefer explicit configuration over magic behavior
6. Ensure all Docker resources are properly labeled for tracking
7. Design for fast startup times
8. Make error messages clear and actionable

## Building and Running

```bash
cargo build          # Build the project
cargo run            # Run the project
cargo test           # Run tests
cargo build --release # Build optimized release binary
```
