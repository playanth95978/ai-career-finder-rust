# Static UI Hosting Guide

This guide explains how to serve the Angular/React/Vue frontend directly from the Rust backend. This enables single-server deployment where both API endpoints and the SPA frontend are served from the same process.

Static file hosting is available for all application types (monolith, gateway, microservice) and can be enabled via environment variables.

## Overview

When enabled, the Rust backend serves:

- **API routes** at `/api/*`, `/management/*`, `/swagger-ui/*`, etc.
- **Static files** (JS, CSS, images) from a configured directory
- **SPA fallback** - All unmatched routes serve `index.html` for client-side routing

This eliminates the need for a separate web server (nginx, Apache) or Node.js process to serve the frontend.

## Configuration

### Environment Variables

Add the following to your `.env` file to enable static file serving:

```env
# Static Files Configuration
# Set to true to enable serving the SPA from the Rust backend
SERVE_STATIC_FILES=true

# Directory containing the built frontend assets
# Relative to the server binary or absolute path
STATIC_FILES_DIR=./dist/static

# Set to true when running behind HTTPS (affects OAuth2 redirect URLs)
APP_HTTPS=false
```

### Configuration Reference

| Variable             | Default    | Description                                                |
| -------------------- | ---------- | ---------------------------------------------------------- |
| `SERVE_STATIC_FILES` | `false`    | Enable/disable static file serving                         |
| `STATIC_FILES_DIR`   | `./static` | Path to directory containing built frontend assets         |
| `APP_HTTPS`          | `false`    | Set to `true` when using HTTPS (for correct redirect URLs) |

### Auto-Detection

If `STATIC_FILES_DIR` is set but `SERVE_STATIC_FILES` is not, static serving is automatically enabled:

```env
# This automatically enables static file serving
STATIC_FILES_DIR=./dist/static
```

## Building the Frontend

### Step 1: Build the SPA

Build your frontend for production:

```bash
cd client
npm run build
```

This creates the production build in `client/dist/<baseName>/browser/` (Angular 17+).

### Step 2: Copy to Static Directory

Copy the built files to where the server expects them:

```bash
# Create the static directory
mkdir -p dist/static

# Copy built assets (Angular)
cp -r client/dist/<baseName>/browser/* dist/static/

# Or for React/Vue, adjust the source path accordingly
```

### Step 3: Configure and Run

```bash
cd server

# Set environment variables
export SERVE_STATIC_FILES=true
export STATIC_FILES_DIR=./dist/static

# Run the server
cargo run
```

Access the application at `http://localhost:8080`.

## SPA Routing Behavior

### How It Works

Single Page Applications use client-side routing. When a user navigates to `/admin/user-management`, that route doesn't exist as a file on the server - it's handled by the Angular/React/Vue router.

The static file handler implements SPA fallback:

1. **Request comes in** for `/admin/user-management`
2. **Check for static file** - Does `/admin/user-management` exist as a file? No.
3. **Serve index.html** - Return `index.html` with status 200
4. **Client router handles it** - Angular Router sees the URL and renders the correct component

### Route Priority

Routes are matched in this order:

| Priority | Route Pattern                      | Handler                   |
| -------- | ---------------------------------- | ------------------------- |
| 1        | `/api/*`                           | API handlers              |
| 2        | `/management/*`                    | Health/info endpoints     |
| 3        | `/swagger-ui/*`                    | Swagger UI                |
| 4        | `/scalar/*`                        | Scalar API docs           |
| 5        | `/oauth2/*`                        | OAuth2 endpoints          |
| 6        | Static files (`.js`, `.css`, etc.) | ServeDir                  |
| 7        | Everything else                    | SPA fallback (index.html) |

### Cache Headers

The server sets appropriate cache headers:

| File Type       | Cache-Control   | Rationale                            |
| --------------- | --------------- | ------------------------------------ |
| `index.html`    | `no-cache`      | Always fetch latest for SPA routing  |
| `*.js`, `*.css` | Browser default | Hashed filenames enable long caching |
| Other assets    | Browser default | Standard caching behavior            |

The `no-cache` header on `index.html` ensures users always get the latest version after deployments, while hashed asset filenames (`main.abc123.js`) allow aggressive browser caching.

## APP_HTTPS Configuration

### Why It Matters

The `APP_HTTPS` setting affects URL generation, particularly for OAuth2 redirects:

```rust
// When APP_HTTPS=false
redirect_uri = "http://localhost:8080/login/oauth2/code/oidc"

// When APP_HTTPS=true
redirect_uri = "https://myapp.example.com/login/oauth2/code/oidc"
```

### When to Enable

| Scenario                          | APP_HTTPS |
| --------------------------------- | --------- |
| Local development                 | `false`   |
| Production with TLS termination   | `true`    |
| Behind a reverse proxy with HTTPS | `true`    |
| Direct HTTPS (rare)               | `true`    |

### TLS Termination

Most production deployments terminate TLS at a load balancer or reverse proxy:

```
Client → HTTPS → Load Balancer → HTTP → Rust Backend
```

In this case, set `APP_HTTPS=true` so the backend generates correct redirect URLs even though it receives HTTP requests.

### Reverse Proxy Configuration

Example nginx configuration:

```nginx
server {
    listen 443 ssl;
    server_name myapp.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Docker Multi-Stage Build

For monolith applications, the generator creates a Dockerfile with a multi-stage build that automatically builds and includes the client. For gateway/microservice applications, static file serving can be enabled at runtime by mounting the built client files.

### Dockerfile Structure

```dockerfile
# Stage 1: Build the frontend
FROM node:22-alpine AS client-builder
WORKDIR /app/client
COPY client/package*.json ./
RUN npm ci
COPY client/ ./
RUN npm run build -- --configuration=production

# Stage 2: Build the Rust backend
FROM rust:1.75-slim AS server-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY server/ ./server/
RUN cargo build --release --manifest-path server/Cargo.toml

# Stage 3: Runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary
COPY --from=server-builder /app/target/release/<app-name> ./

# Copy the built frontend
COPY --from=client-builder /app/client/dist/<baseName>/browser ./static/

# Configure for static serving
ENV SERVE_STATIC_FILES=true
ENV STATIC_FILES_DIR=./static
ENV APP_PORT=8080

EXPOSE 8080
CMD ["./<app-name>"]
```

### Build and Run

```bash
# Build the image
docker build -t myapp:latest .

# Run the container
docker run -d \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://... \
  -e JWT_SECRET=your-secret \
  myapp:latest
```

### Image Size Optimization

The multi-stage build produces a minimal image:

| Stage          | Purpose                    | Not included in final |
| -------------- | -------------------------- | --------------------- |
| client-builder | Node.js, npm, source       | ✓                     |
| server-builder | Rust toolchain, source     | ✓                     |
| runtime        | Binary + static files only | -                     |

Typical final image size: ~100-150MB (depending on static assets).

## Docker Compose

For development with Docker Compose:

```yaml
# docker-compose.yml
services:
  app:
    build: .
    ports:
      - '8080:8080'
    environment:
      - DATABASE_URL=postgres://postgres:postgres@db:5432/myapp
      - JWT_SECRET=dev-secret-change-in-production
      - SERVE_STATIC_FILES=true
      - STATIC_FILES_DIR=./static
    depends_on:
      - db

  db:
    image: postgres:16
    environment:
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=myapp
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

## OAuth2/Keycloak Considerations

When using OAuth2 authentication with static file serving:

### Redirect URI Configuration

The redirect URI must be registered in your identity provider. With monolithic deployment, both frontend and backend share the same origin:

```
# Keycloak client configuration
Valid Redirect URIs: http://localhost:8080/*
                     https://myapp.example.com/*
```

### Post-Login Redirect

After OAuth2 authentication, users are redirected back to the application:

1. User clicks login → Redirected to Keycloak
2. User authenticates → Keycloak redirects to `/login/oauth2/code/oidc`
3. Backend exchanges code for token → Redirects to frontend URL
4. Frontend loads with authentication cookie set

### Frontend URL

With `SERVE_STATIC_FILES=true`, the backend redirects to itself after authentication (not a separate frontend URL).

## Troubleshooting

### Static Files Not Served

**Symptom**: 404 errors for static assets

**Solutions**:

1. Verify `SERVE_STATIC_FILES=true` is set
2. Check `STATIC_FILES_DIR` points to correct directory
3. Ensure the directory contains built assets:
   ```bash
   ls -la ./dist/static/
   # Should show index.html, main.*.js, etc.
   ```

### index.html Not Found

**Symptom**: "Application not found. Please build the client first."

**Solutions**:

1. Build the frontend: `cd client && npm run build`
2. Copy assets to static directory
3. Verify `index.html` exists in `STATIC_FILES_DIR`

### SPA Routes Return 404

**Symptom**: Direct navigation to `/admin/users` returns 404

**Solutions**:

1. Ensure static file serving is enabled
2. Check that the route isn't conflicting with an API route
3. Verify `index.html` is being served (check network tab)

### OAuth2 Redirect Fails

**Symptom**: "Invalid redirect URI" error from Keycloak

**Solutions**:

1. Set `APP_HTTPS=true` if using HTTPS
2. Register the correct redirect URI in Keycloak
3. Check `FRONTEND_URL` environment variable (OAuth2 mode)

### Assets Load Over Wrong Protocol

**Symptom**: Mixed content warnings, assets trying to load over HTTP

**Solutions**:

1. Set `APP_HTTPS=true` when behind HTTPS proxy
2. Configure proxy to set `X-Forwarded-Proto` header
3. Ensure frontend build uses relative paths

## Development vs Production

| Setting              | Development       | Production                |
| -------------------- | ----------------- | ------------------------- |
| `SERVE_STATIC_FILES` | `true` (optional) | `true`                    |
| `STATIC_FILES_DIR`   | `./dist/static`   | `./static` (in container) |
| `APP_HTTPS`          | `false`           | `true`                    |
| `APP_ENV`            | `development`     | `production`              |

### Development Workflow

For development, you can either:

**Option 1: Separate servers (recommended for hot reload)**

```bash
# Terminal 1: Frontend with hot reload
cd client && npm start  # Runs on port 9000

# Terminal 2: Backend API only
cd server && cargo run  # Runs on port 8080
```

**Option 2: Monolithic (tests production setup)**

```bash
# Build frontend
cd client && npm run build

# Copy and run
mkdir -p ../dist/static
cp -r dist/<baseName>/browser/* ../dist/static/
cd ../server
SERVE_STATIC_FILES=true STATIC_FILES_DIR=../dist/static cargo run
```

## Security Considerations

### Path Traversal Protection

The `tower-http` `ServeDir` service handles path sanitization automatically, preventing directory traversal attacks like `/../../../etc/passwd`.

### No Directory Listing

Directory browsing is disabled by default. Requests to directories without an index file return the SPA fallback.

### Content-Type Headers

Static files are served with correct MIME types based on file extensions, preventing content-type confusion attacks.

### Recommended Headers

For production, consider adding security headers via your reverse proxy:

```nginx
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
```

## Future Enhancement: Embedded Assets

For single-binary distribution without external static files, a future enhancement could use `rust-embed` to compile assets into the binary:

```rust
#[derive(rust_embed::RustEmbed)]
#[folder = "static/"]
struct Assets;
```

This would eliminate the need for a separate static directory but increase binary size. This feature is not yet implemented.
