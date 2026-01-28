# Configuration Module

## Location
[src/config/mod.rs](src/config/mod.rs)

## Philosophy
- Simple por default, personalizable cuando se necesita
- Formato corto para casos comunes
- Formato largo para configuración avanzada
- Env vars con defaults sensatos para DBs

## Configuration Schema

### Formato Corto
```yaml
name: my-app
redis: 7
node: 20
```

### Formato Largo
```yaml
name: my-app

mysql:
  version: 8.0
  port: 3306
  image: mysql:8.0          # opcional
  env:                      # opcional
    MYSQL_DATABASE: mydb
    MYSQL_USER: admin
```

## Imágenes Oficiales Default

| Servicio | Imagen Default |
|----------|----------------|
| node | `node:{version}-alpine` |
| redis | `redis:{version}-alpine` |
| php | `php:{version}-fpm-alpine` |
| mysql | `mysql:{version}` |
| postgres | `postgres:{version}-alpine` |
| mongodb | `mongo:{version}` |
| nginx | `nginx:{version}-alpine` |
| python | `python:{version}-slim` |
| ruby | `ruby:{version}-slim` |
| golang | `golang:{version}-alpine` |

## Env Vars Default

Foundry agrega env vars automáticamente para DBs:

| Servicio | Variables Default |
|----------|-------------------|
| mysql | `MYSQL_ROOT_PASSWORD=secret`, `MYSQL_DATABASE=app` |
| postgres | `POSTGRES_PASSWORD=secret`, `POSTGRES_DB=app` |
| mongodb | `MONGO_INITDB_ROOT_USERNAME=root`, `MONGO_INITDB_ROOT_PASSWORD=secret` |

Puedes sobrescribir cualquier variable en el config:
```yaml
mysql:
  version: 8.0
  env:
    MYSQL_ROOT_PASSWORD: my_secure_password
    MYSQL_DATABASE: production_db
```

## Rust Structures

```rust
#[serde(untagged)]
pub enum ServiceSpec {
    ShortNum(u32),           // redis: 7
    ShortStr(String),        // node: "20"
    Long(ServiceConfig),     // mysql: { version: 8.0, port: 3306 }
}

pub struct ServiceConfig {
    pub version: Option<serde_yaml::Value>,
    pub port: Option<u16>,
    pub image: Option<String>,
    pub env: HashMap<String, String>,
}

pub struct Service {
    pub name: String,
    pub image: String,
    pub version: String,
    pub port: Option<u16>,
    pub env: HashMap<String, String>,
}
```

## API

```rust
let config = Config::load()?;
let services = config.get_services()?;

for service in services {
    println!("{}: {} (port {:?})", service.name, service.image, service.port);
    for (k, v) in &service.env {
        println!("  {}={}", k, v);
    }
}
```
