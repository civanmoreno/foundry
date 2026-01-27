# Test Examples

## Location
[test/](test/)

## Purpose
Contains example configurations for testing Foundry functionality.

## Example Project: my-app

Located in [test/foundry.yaml](test/foundry.yaml)

A typical web application stack with:
- **app** - Node.js application server
- **db** - PostgreSQL database
- **redis** - Redis cache

### Usage

```bash
cd test
foundry up        # Start all services
foundry status    # Check running services
foundry exec app npm install
foundry logs app --follow
foundry down      # Stop services
foundry clean     # Remove all resources
```

### Service Dependencies

```
app → db, redis
```

The `app` service waits for `db` and `redis` to be ready before starting.

## Creating New Test Configs

When adding test configurations:
1. Create a new directory or use existing `test/`
2. Add a `foundry.yaml` with the service definitions
3. Keep examples realistic but minimal
4. Document environment variables and ports used
