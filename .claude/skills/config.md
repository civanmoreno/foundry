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

## PHP Extensions

Foundry construye automáticamente una imagen custom de PHP cuando se especifican extensiones:

```yaml
php:
  version: 8.3
  extensions:
    - pdo_mysql
    - redis
    - gd
```

### Extensiones Built-in (docker-php-ext-install)
- `pdo_mysql`, `pdo_pgsql`, `pdo_sqlite`
- `mysqli`, `pgsql`
- `gd`, `intl`, `zip`, `soap`, `xsl`
- `bcmath`, `opcache`, `pcntl`

### Extensiones PECL (pecl install)
- `redis`, `xdebug`, `imagick`
- `memcached`, `mongodb`, `apcu`

Foundry instala automáticamente las dependencias del sistema necesarias (libpng-dev, icu-dev, etc.) según las extensiones solicitadas.

## Root Path (Volume Mount)

### Global Root
Define el directorio host a montar en todos los servicios:
```yaml
name: my-app
root: ./public    # relativo al foundry.yaml
```

Soporta paths absolutos y relativos:
- `root: ./public` → resuelve a `/path/to/project/public`
- `root: /var/www/html` → usa el path absoluto
- Sin `root` → usa el directorio actual (donde está foundry.yaml)

### Destinos por Servicio
Foundry monta automáticamente el directorio del proyecto en servicios que lo necesitan:

| Servicio | Destino en Container |
|----------|---------------------|
| php | `/var/www/html` |
| nginx | `/usr/share/nginx/html` |
| node | `/app` |
| python | `/app` |
| ruby | `/app` |
| golang | `/app` |

### Custom Root por Servicio
Puedes cambiar el destino dentro del container:
```yaml
php:
  version: 8.3
  root: /custom/path    # destino en el container
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
pub struct Config {
    pub name: String,
    pub root: Option<String>,    // global root path
    pub services: HashMap<String, ServiceSpec>,
}

#[serde(untagged)]
pub enum ServiceSpec {
    ShortNum(u32),           // redis: 7
    ShortFloat(f64),         // php: 8.3
    ShortStr(String),        // node: "20"
    Long(ServiceConfig),     // mysql: { version: 8.0, port: 3306 }
}

pub struct ServiceConfig {
    pub version: Option<serde_yaml::Value>,
    pub port: Option<u16>,
    pub image: Option<String>,
    pub env: HashMap<String, String>,
    pub root: Option<String>,    // container destination path
    pub extensions: Vec<String>, // PHP extensions
}

pub struct Service {
    pub name: String,
    pub image: String,
    pub version: String,
    pub port: Option<u16>,
    pub env: HashMap<String, String>,
    pub root: Option<String>,    // container destination path
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
