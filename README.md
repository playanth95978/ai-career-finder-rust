# jobSearchRust

This application was generated using JHipster 9.2.0 with the Rust blueprint.

## Technology Stack

- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **ORM**: [Diesel](https://diesel.rs/)
- **Database**: PostgreSQL
- **Runtime**: [Tokio](https://tokio.rs/)
- **Authentication**: JWT

## Development

### Prerequisites

- Rust 1.75 or later
- PostgreSQL 14 or later
- PostgreSQL client library (`libpq`):
  - macOS: `brew install libpq && echo 'export LIBRARY_PATH="/opt/homebrew/opt/libpq/lib:$LIBRARY_PATH"' >> ~/.zshrc`
  - Ubuntu/Debian: `sudo apt-get install libpq-dev`
  - Fedora: `sudo dnf install libpq-devel`
- Diesel CLI: `cargo install diesel_cli --no-default-features --features postgres`

### Setup

1. Start PostgreSQL using Docker:

   ```bash
   docker-compose up -d
   ```

2. Run migrations:

   ```bash
   diesel migration run
   ```

3. Start the application:
   ```bash
   cargo run
   ```

The server will start at `http://localhost:8080`.

### Available Endpoints

- `GET /api/health` - Health check
- `GET /api/users` - List users (authenticated)
- `GET /api/account` - Get current user account (authenticated)
- `POST /api/authenticate` - Login

## Building for Production

```bash
cargo build --release
```

## Docker

Build and run with Docker:

```bash
docker build -t jobSearchRust .
docker run -p 8080:8080 jobSearchRust
```

## Testing

```bash
cargo test
```

For database tests, use single-threaded execution to avoid migration conflicts:

```bash
cargo test -- --test-threads=1
```

## CI/CD

Generate CI/CD configuration for your project:

```bash
jhipster-rust ci-cd
```

This supports:

- **GitHub Actions** - `.github/workflows/main.yml`
- **GitLab CI** - `.gitlab-ci.yml`

### Running CI Locally

You can run GitHub Actions locally using [act](https://github.com/nektos/act):

```bash
# Install act (macOS)
brew install act

# Run the workflow
act push --container-architecture linux/amd64
```

See [CI/CD Documentation](docs/CI_CD.md) for more details.

## Project Structure

```
.
├── Cargo.toml              # Workspace manifest
├── diesel.toml             # Diesel configuration
├── docker-compose.yml      # Docker Compose for PostgreSQL
├── migrations/             # Database migrations
└── server/
    ├── Cargo.toml          # Server crate manifest
    └── src/
        ├── main.rs         # Application entry point
        ├── lib.rs          # Library root
        ├── config/         # Configuration
        ├── db/             # Database connection
        ├── models/         # Diesel models
        ├── handlers/       # Axum route handlers
        ├── services/       # Business logic
        ├── middleware/     # Middleware (auth, etc.)
        ├── errors/         # Error types
        └── dto/            # Data transfer objects
```

## Documentation

Detailed documentation is available in the `docs/` directory:

- [Configuration Reference](docs/CONFIG.md) - All environment variables, external config, and production checklist
- [CI/CD Integration](docs/CI_CD.md) - GitHub Actions, GitLab CI, running locally with act
- [Docker Integration](docs/DOCKER.md) - Container configuration and deployment
- [Testing Guide](docs/TESTING.md) - Unit tests, integration tests, and test utilities
- [Security Guide](docs/SECURITY.md) - Authentication, authorization, and security best practices
- [Entity Generation](docs/ENTITY_GENERATION.md) - Adding new entities to your application
- [OpenAPI/Swagger](docs/OPENAPI.md) - API documentation and Swagger UI
- [PostgreSQL Setup](docs/POSTGRES.md) - Database configuration and management
- [Static File Hosting](docs/STATIC_HOSTING.md) - Serving frontend assets
