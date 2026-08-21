# Configuration Reference

This document describes all configuration options for **jobSearchRust**, organized by feature. Configuration is managed through environment variables in the `.env` file, with optional layered configuration via Consul KV when external configuration is enabled.

## Configuration Layers

Configuration is loaded from the following sources (highest precedence wins):

1. **Environment variables** (always take precedence)
2. **`.env` file** (loaded at startup via `dotenvy`)
3. **Compiled defaults** (hardcoded fallbacks in the application)

## Application Settings

Core application settings that control the server behavior.

| Variable             | Description                                               | Default                                     |
| -------------------- | --------------------------------------------------------- | ------------------------------------------- |
| `APP_NAME`           | Application name (used in logs and metadata)              | `jobSearchRust`                             |
| `APP_ENV`            | Environment name (`development`, `staging`, `production`) | `development`                               |
| `APP_PORT`           | HTTP server port                                          | `8080`                                      |
| `APP_HOST`           | Bind address for the HTTP server                          | `0.0.0.0`                                   |
| `APP_HTTPS`          | Set to `true` when running behind HTTPS                   | `false`                                     |
| `SERVE_STATIC_FILES` | Enable serving the SPA UI from Rust backend               | `false`                                     |
| `STATIC_FILES_DIR`   | Directory containing the built frontend app               | `./static`                                  |
| `RUST_LOG`           | Logging level filter (uses `tracing` format)              | `info,jobSearchRust=debug,tower_http=debug` |

## Database

| Variable       | Description                  | Default                                                     |
| -------------- | ---------------------------- | ----------------------------------------------------------- |
| `DATABASE_URL` | PostgreSQL connection string | `postgres://postgres:postgres@localhost:5432/jobsearchrust` |

## Authentication

JWT-based stateless authentication.

| Variable               | Description                                                           | Default        |
| ---------------------- | --------------------------------------------------------------------- | -------------- |
| `JWT_SECRET`           | Secret key for signing JWT tokens. **Must be changed in production.** | Auto-generated |
| `JWT_EXPIRATION_HOURS` | Token expiration time in hours                                        | `24`           |

## Production Checklist

Before deploying to production, review these critical settings:

- [ ] **`JWT_SECRET`**: Replace with a strong, random secret (minimum 256 bits)
- [ ] **`APP_ENV`**: Set to `production`
- [ ] **`DATABASE_URL`**: Update with production database credentials
- [ ] **`RUST_LOG`**: Set to `info` or `warn` (avoid `debug` in production)
