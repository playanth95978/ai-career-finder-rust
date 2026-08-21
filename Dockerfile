# =============================================================================
# Client Build Stage (for serving SPA UI from Rust backend)
# =============================================================================
# This stage builds the Angular/React/Vue client for production.
# The built client files are copied to the runtime stage.
FROM node:22-alpine AS client-builder

WORKDIR /app/client

# Copy package file before installing dependencies (cached layer).
COPY client/package.json ./
# Angular projects ship a client/scripts/copy-swagger-ui-assets.cjs helper that
# is invoked by the `postinstall` hook in client/package.json (it vendors the
# swagger-ui-dist runtime files into client/src/swagger-ui/ so the Vite-based
# Angular dev server can serve them in dev mode). The helper file must exist
# at install time, otherwise `npm install` aborts with `Cannot find module`.
COPY client/scripts ./scripts
RUN npm install --force

# Copy client source and build for production.
# Remove ESLintWebpackPlugin from webpack config — linting is done during
# development, not needed in the production Docker image build.
COPY client/ ./
RUN if [ -f webpack/webpack.common.js ]; then \
      node -e " \
        let f=require('fs').readFileSync('webpack/webpack.common.js','utf8'); \
        f=f.replace(/const ESLintPlugin.*?\n/,''); \
        f=f.replace(/new ESLintPlugin\(\{[^}]*\}\),?\s*/g,''); \
        require('fs').writeFileSync('webpack/webpack.common.js',f);" ; \
    fi && \
    npm run webapp:prod

# =============================================================================
# Server Build Stage
# =============================================================================
FROM rust:1.89-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml ./
COPY server/Cargo.toml ./server/

# Create dummy source to cache dependencies
RUN mkdir -p server/src && \
    echo "fn main() {}" > server/src/main.rs && \
    cargo build --release && \
    rm -rf server/src

# Copy actual source code
COPY server/src ./server/src
COPY migrations ./migrations

# Build the application
RUN touch server/src/main.rs && \
    cargo build --release

# =============================================================================
# Runtime Stage
# =============================================================================
FROM debian:trixie-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary (Rust uses snake_case for binary names)
COPY --from=builder /app/target/release/job_search_rust /app/job_search_rust
COPY --from=builder /app/migrations /app/migrations

# Copy the built client for serving SPA UI from Rust backend
COPY --from=client-builder /app/server/dist/static/ ./static/

# Application configuration
ENV APP_NAME=job_search_rust
ENV APP_ENV=production
ENV APP_PORT=8080
ENV APP_HOST=0.0.0.0

# DATABASE_URL / MONGODB_URI are NOT baked into the image. The application
# refuses to start without them so operators are forced to make a deliberate
# choice rather than inherit a known-default credential from the template.
# Examples:
#   docker run -e DATABASE_URL=postgres://user:pass@host:5432/db ...
#   docker run -e MONGODB_URI=mongodb://user:pass@host:27017 ...
#   docker run -e DATABASE_URL=sqlite:///app/target/db/job_search_rust.db -v <vol>:/app/target/db ...
# See RELEASE_NOTES.md for migration guidance.

# JWT configuration
# JWT_SECRET is generated at container start by docker-entrypoint.sh when unset.
# To pin a fixed secret across container restarts (recommended for production),
# supply it explicitly: docker run -e JWT_SECRET="$(openssl rand -hex 32)" ...
ENV JWT_EXPIRATION_HOURS=24

# Static file serving configuration (SPA UI served from Rust backend)
ENV SERVE_STATIC_FILES=true
ENV STATIC_FILES_DIR=./static

COPY docker-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Run as a non-root user for defense-in-depth (closes /cso Finding #8).
# UID 1001 must match the K8s Deployment's securityContext.runAsUser.
RUN useradd --system --uid 1001 --no-create-home appuser
RUN chown -R 1001:1001 /app
USER 1001

EXPOSE 8080

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/app/job_search_rust"]
