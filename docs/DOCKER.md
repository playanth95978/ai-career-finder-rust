# Docker Integration

This document covers Docker configuration, container management, and deployment options for JHipster Rust applications.

## Overview

The generator creates Docker configurations for:

- **Application container**: Multi-stage Rust build with optimized runtime
- **Database containers**: PostgreSQL, MySQL, MongoDB (SQLite runs without a container)
- **Authentication containers**: Keycloak for OAuth2/OIDC

## Directory Structure

Generated Docker files are located in the `docker/` directory:

```
your-app/
├── Dockerfile                    # Application container
├── docker/
│   ├── app.yml                   # Full stack (app + database)
│   ├── keycloak.yml              # Keycloak OAuth2 (if OAuth2 enabled)
│   └── realm-config/
│       └── jhipster-realm.json   # Keycloak realm configuration
└── docker-compose.yml            # Database only (PostgreSQL)
```

## Quick Start

### Starting the Full Stack

To start the application with all dependencies:

```bash
# Build and start everything
docker compose -f docker/app.yml up -d

# View logs
docker compose -f docker/app.yml logs -f

# Stop everything
docker compose -f docker/app.yml down
```

### Starting Individual Services

For development, you may want to run only specific services:

```bash
# Start only the database
docker compose -f docker/app.yml up -d db

# Start only Keycloak (OAuth2 projects)
docker compose -f docker/keycloak.yml up -d

# Then run the application locally
cd server && cargo run
```

## Application Container

### Dockerfile

The generated Dockerfile uses a multi-stage build for optimal image size:

**Build Stage:**

- Base: `rust:1.89-slim`
- Installs database-specific build dependencies
- Caches Cargo dependencies for faster rebuilds
- Compiles release binary

**Runtime Stage:**

- Base: `debian:bookworm-slim`
- Minimal runtime dependencies
- Copies only the compiled binary
- Exposes port 8080

### Building the Application Image

```bash
# Build the image
docker build -t your-app .

# Run the container
docker run -p 8080:8080 \
  -e DATABASE_URL="postgres://postgres:postgres@host.docker.internal:5432/yourapp" \
  -e JWT_SECRET="your-secret-key" \
  your-app
```

### Environment Variables

| Variable               | Default      | Description                          |
| ---------------------- | ------------ | ------------------------------------ |
| `APP_NAME`             | `{appName}`  | Application name                     |
| `APP_ENV`              | `production` | Environment (development/production) |
| `APP_PORT`             | `8080`       | Server port                          |
| `APP_HOST`             | `0.0.0.0`    | Server bind address                  |
| `DATABASE_URL`         | varies       | Database connection string           |
| `JWT_SECRET`           | -            | JWT signing secret (required)        |
| `JWT_EXPIRATION_HOURS` | `24`         | Token expiration                     |
| `RUST_LOG`             | `info`       | Logging level                        |

## Database Containers

### PostgreSQL

**Container Details:**

- Image: `postgres:16-alpine`
- Port: `5432`
- Default credentials: `postgres:postgres`

**Starting PostgreSQL:**

```bash
# Using app.yml (includes health checks)
docker compose -f docker/app.yml up -d db

# Or using the standalone docker-compose.yml
docker compose up -d
```

**Connection String:**

```
postgres://postgres:postgres@localhost:5432/{appname}
```

**Accessing PostgreSQL:**

```bash
# Connect with psql
docker exec -it {appname}-postgresql psql -U postgres -d {appname}

# Common commands
\dt          # List tables
\d users     # Describe table
\q           # Quit
```

**Data Persistence:**

PostgreSQL data is stored in a named volume:

```yaml
volumes:
  postgresql-data:
```

To reset the database:

```bash
docker compose -f docker/app.yml down -v  # -v removes volumes
docker compose -f docker/app.yml up -d db
```

### MySQL

**Container Details:**

- Image: `mysql:8.0`
- Port: `3306`
- Default credentials: `root:root`

**Starting MySQL:**

```bash
docker compose -f docker/app.yml up -d db
```

**Connection String:**

```
mysql://root:root@127.0.0.1:3306/{appname}
```

**Accessing MySQL:**

```bash
# Connect with mysql client
docker exec -it {appname}-mysql mysql -u root -proot {appname}

# Common commands
SHOW TABLES;
DESCRIBE users;
EXIT;
```

**Data Persistence:**

MySQL data is stored in a named volume:

```yaml
volumes:
  mysql-data:
```

### SQLite

SQLite doesn't require a separate container. The database file is stored locally:

**Development:**

```
sqlite://./target/db/{appname}.db
```

**Docker (app container):**

```
sqlite:///app/target/db/{appname}.db
```

The Dockerfile creates the necessary directory:

```dockerfile
RUN mkdir -p /app/target/db
```

### MongoDB

**Container Details:**

- Image: `mongo:7.0`
- Port: `27017`
- Default credentials: `root:secret`
- Configuration: Replica set (required for transactions)

**Starting MongoDB:**

```bash
docker compose up -d
```

**Initializing the Replica Set (first time only):**

```bash
docker exec -it {appname}-mongodb mongosh --eval "rs.initiate()"
```

**Connection String:**

```
mongodb://root:secret@localhost:27017
```

**Environment Variables:**

```env
MONGODB_URI=mongodb://root:secret@localhost:27017
MONGODB_DATABASE={appname}
```

**Accessing MongoDB:**

```bash
# Connect with mongosh
docker exec -it {appname}-mongodb mongosh -u root -p secret --authenticationDatabase admin

# Common commands
show dbs           # List databases
use {appname}      # Switch database
show collections   # List collections
db.users.find()    # Query documents
exit               # Quit
```

**Data Persistence:**

MongoDB data is stored in a named volume:

```yaml
volumes:
  mongodb-data:
```

To reset the database:

```bash
docker compose down -v  # -v removes volumes
docker compose up -d
# Re-initialize replica set
docker exec -it {appname}-mongodb mongosh --eval "rs.initiate()"
```

**Why Replica Set?**

MongoDB transactions require a replica set. The generator configures a single-node replica set for development. This provides:

- Transaction support for multi-document operations
- Change streams capability
- Production-like behavior

For detailed MongoDB setup, see [MongoDB Integration](MONGODB.md).

## Keycloak Container

For OAuth2/OIDC authentication, the generator creates a Keycloak configuration.

**Container Details:**

- Image: `quay.io/keycloak/keycloak:23.0`
- Port: `9080`
- Admin credentials: `admin:admin`

**Starting Keycloak:**

```bash
docker compose -f docker/keycloak.yml up -d
```

**Access URLs:**

- Admin Console: http://localhost:9080/admin
- Realm: http://localhost:9080/realms/jhipster

**Pre-configured Realm:**

The `jhipster` realm is automatically imported with:

| Setting       | Value             |
| ------------- | ----------------- |
| Realm         | `jhipster`        |
| Client ID     | `web_app`         |
| Client Secret | `web_app`         |
| Admin User    | `admin` / `admin` |
| Test User     | `user` / `user`   |

**Roles:**

- `ROLE_ADMIN` - Administrator access
- `ROLE_USER` - Standard user access

For detailed Keycloak configuration, see [Keycloak Integration](KEYCLOAK.md).

## Docker Compose Files

### app.yml - Full Stack

Starts the application with its database:

```yaml
services:
  app:
    build: ..
    ports:
      - '127.0.0.1:8080:8080'
    environment:
      - DATABASE_URL=postgres://postgres:postgres@db:5432/{appname}
    depends_on:
      db:
        condition: service_healthy

  db:
    image: postgres:16-alpine
    ports:
      - '5432:5432'
    volumes:
      - postgresql-data:/var/lib/postgresql/data
```

**Usage:**

```bash
# Start all services
docker compose -f docker/app.yml up -d

# Rebuild application after code changes
docker compose -f docker/app.yml up -d --build

# View application logs
docker compose -f docker/app.yml logs -f app

# Stop and remove containers
docker compose -f docker/app.yml down
```

### keycloak.yml - OAuth2 Provider

Starts Keycloak with pre-configured realm:

```yaml
services:
  keycloak:
    image: quay.io/keycloak/keycloak:23.0
    ports:
      - '9080:8080'
    environment:
      - KEYCLOAK_ADMIN=admin
      - KEYCLOAK_ADMIN_PASSWORD=admin
    volumes:
      - ./realm-config:/opt/keycloak/data/import
    command: start-dev --import-realm
```

**Usage:**

```bash
# Start Keycloak
docker compose -f docker/keycloak.yml up -d

# View logs
docker compose -f docker/keycloak.yml logs -f

# Stop Keycloak
docker compose -f docker/keycloak.yml down
```

## Health Checks

All services include health checks for reliable startup ordering:

| Service    | Health Check         | Interval | Retries |
| ---------- | -------------------- | -------- | ------- |
| App        | `GET /api/health`    | 5s       | 40      |
| PostgreSQL | `pg_isready`         | 5s       | 10      |
| MySQL      | `mysqladmin ping`    | 5s       | 10      |
| Keycloak   | HTTP health endpoint | 10s      | 20      |

The application waits for the database to be healthy before starting:

```yaml
depends_on:
  db:
    condition: service_healthy
```

## Common Commands

### Container Management

```bash
# List running containers
docker compose -f docker/app.yml ps

# Stop all containers
docker compose -f docker/app.yml stop

# Start stopped containers
docker compose -f docker/app.yml start

# Restart a specific service
docker compose -f docker/app.yml restart app

# Remove containers (keeps volumes)
docker compose -f docker/app.yml down

# Remove containers and volumes (full reset)
docker compose -f docker/app.yml down -v
```

### Logs and Debugging

```bash
# View all logs
docker compose -f docker/app.yml logs

# Follow logs in real-time
docker compose -f docker/app.yml logs -f

# View logs for specific service
docker compose -f docker/app.yml logs -f app

# View last 100 lines
docker compose -f docker/app.yml logs --tail=100 app
```

### Executing Commands

```bash
# Run shell in app container
docker compose -f docker/app.yml exec app /bin/bash

# Run database migrations
docker compose -f docker/app.yml exec app diesel migration run

# Connect to PostgreSQL
docker compose -f docker/app.yml exec db psql -U postgres -d {appname}

# Connect to MySQL
docker compose -f docker/app.yml exec db mysql -u root -proot {appname}
```

## Development Workflow

### Option 1: Database in Docker, App Local

Best for active development with fast iteration:

```bash
# Start only the database
docker compose -f docker/app.yml up -d db

# Run migrations
cd server && diesel migration run

# Run the application locally
cargo run
```

### Option 2: Full Stack in Docker

Best for testing the complete containerized setup:

```bash
# Build and start everything
docker compose -f docker/app.yml up -d --build

# Test the application
curl http://localhost:8080/api/health
```

### Option 3: OAuth2 Development

For OAuth2 projects, run Keycloak separately:

```bash
# Terminal 1: Start Keycloak
docker compose -f docker/keycloak.yml up -d

# Terminal 2: Start database
docker compose -f docker/app.yml up -d db

# Terminal 3: Run application
cd server && cargo run
```

## Monolithic Deployment

For monolithic applications, the Dockerfile includes a multi-stage build that compiles both the Angular client and Rust server into a single container. This enables single-container deployment where the Rust backend serves both API endpoints and the Angular frontend.

### How It Works

The Dockerfile uses three stages:

1. **Client Builder Stage** (`node:22-alpine`): Builds the Angular application
2. **Server Builder Stage** (`rust:1.89-slim`): Compiles the Rust backend
3. **Runtime Stage** (`debian:bookworm-slim`): Combines both into a minimal runtime image

### Building the Monolithic Image

```bash
# Build the complete monolithic image
docker build -t your-app .

# Run the container
docker run -p 8080:8080 \
  -e DATABASE_URL="sqlite:///app/target/db/yourapp.db" \
  your-app
```

### Environment Variables for Monolithic Mode

The monolithic container automatically sets:

| Variable             | Value      | Description                            |
| -------------------- | ---------- | -------------------------------------- |
| `SERVE_STATIC_FILES` | `true`     | Enables static file serving            |
| `STATIC_FILES_DIR`   | `./static` | Directory containing the Angular build |

### With Docker Compose

```bash
# Build and run with database
docker compose -f docker/app.yml up -d --build
```

The application will be available at http://localhost:8080, serving both:

- API endpoints at `/api/*`
- Angular UI for all other routes

### OAuth2/Keycloak with Monolithic Docker

When using OAuth2 authentication with the monolithic container:

1. Ensure Keycloak is running and configured
2. Set the redirect URI in Keycloak to `http://localhost:8080/*`
3. For HTTPS in production, set `APP_HTTPS=true`:

```bash
docker run -p 8080:8080 \
  -e APP_HTTPS=true \
  -e OIDC_ISSUER_URI="https://your-keycloak/realms/jhipster" \
  your-app
```

## Production Considerations

### Security

1. **Change default credentials:**

   ```yaml
   environment:
     - POSTGRES_PASSWORD=strong-random-password
     - JWT_SECRET=cryptographically-secure-secret
   ```

2. **Don't expose database ports:**

   ```yaml
   # Remove or bind to localhost only
   ports:
     - '127.0.0.1:5432:5432'
   ```

3. **Use secrets management:**
   ```yaml
   secrets:
     db_password:
       external: true
   ```

### Performance

1. **Resource limits:**

   ```yaml
   services:
     app:
       deploy:
         resources:
           limits:
             cpus: '2'
             memory: 512M
   ```

2. **Connection pooling:**
   The application uses r2d2 connection pooling with sensible defaults.

### Persistence

1. **Named volumes for data:**

   ```yaml
   volumes:
     postgresql-data:
       driver: local
   ```

2. **Backup strategy:**

   ```bash
   # PostgreSQL backup
   docker exec {appname}-postgresql pg_dump -U postgres {appname} > backup.sql

   # MySQL backup
   docker exec {appname}-mysql mysqldump -u root -proot {appname} > backup.sql
   ```

## Troubleshooting

### Container Won't Start

**Check logs:**

```bash
docker compose -f docker/app.yml logs app
```

**Common issues:**

- Database not ready: Wait for health check or check database logs
- Port already in use: Change port mapping or stop conflicting service
- Missing environment variable: Check required variables are set

### Database Connection Failed

**Verify database is running:**

```bash
docker compose -f docker/app.yml ps db
```

**Check connectivity:**

```bash
# PostgreSQL
docker compose -f docker/app.yml exec db pg_isready

# MySQL
docker compose -f docker/app.yml exec db mysqladmin ping -u root -proot
```

**Verify DATABASE_URL:**

- Inside Docker network: Use `db` as hostname
- From host machine: Use `localhost`

### Build Failures

**Clear Docker cache:**

```bash
docker compose -f docker/app.yml build --no-cache
```

**Check disk space:**

```bash
docker system df
docker system prune  # Clean up unused resources
```

### Keycloak Issues

**Realm not imported:**

```bash
# Check Keycloak logs
docker compose -f docker/keycloak.yml logs keycloak

# Verify realm file exists
ls docker/realm-config/jhipster-realm.json
```

**Can't access admin console:**

- URL: http://localhost:9080/admin
- Credentials: admin / admin
- Wait for health check to pass (can take 30-60 seconds)
