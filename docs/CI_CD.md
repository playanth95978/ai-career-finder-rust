# CI/CD Integration

This document covers CI/CD configuration for jobSearchRust.

## Overview

CI/CD pipelines can be generated using `jhipster-rust ci-cd` and are configured for:

- **Building** the Rust backend with `cargo build --release`
- **Testing** with `cargo test`
- **Linting** with `cargo clippy`
- **Docker image publishing** (optional)
- **SonarQube code analysis** (optional)

## Running CI/CD Locally

You can run GitHub Actions locally using [act](https://github.com/nektos/act), which simulates the GitHub Actions environment in Docker containers.

### Installing Act

```bash
# macOS
brew install act

# Linux
curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Windows (with Chocolatey)
choco install act-cli
```

### Running the Workflow

```bash
# For Apple Silicon Macs (M1/M2/M3/M4)
act push --container-architecture linux/amd64

# For Intel Macs or Linux
act push
```

### Other Useful Commands

```bash
# Dry run (shows what would run without executing)
act -n

# Run a specific job
act push -j build --container-architecture linux/amd64

# List available workflows and jobs
act -l
```

### First Run Setup

On first run, `act` will prompt you to select a Docker image size:

- **Micro** - Minimal image, faster but may lack some tools
- **Medium** - Good balance (recommended)
- **Large** - Full GitHub Actions environment, slower to download

### Important Notes

- The `--container-architecture linux/amd64` flag is required on Apple Silicon Macs
- `act` mounts your local directory into the container
- First run will download Docker images which may take some time
- Some GitHub Actions features (like secrets) need separate configuration

## GitHub Actions Workflow

When you run `jhipster-rust ci-cd` and select GitHub Actions, a workflow is generated at `.github/workflows/main.yml`:

### Build Job

```yaml
jobs:
  build:
    - Checkout code
    - Setup Rust toolchain (stable)
    - Cache Cargo dependencies
    - Run cargo clippy (linting)
    - Run cargo build --release
    - Run cargo test
```

### Database Service

PostgreSQL is configured as a service container:

```yaml
services:
  postgres:
    image: postgres:15
    env:
      POSTGRES_USER: jobSearchRust
      POSTGRES_PASSWORD: jobSearchRust
      POSTGRES_DB: jobSearchRust
    ports:
      - 5432:5432
```

### Docker Publishing (Optional)

When enabled, a separate job builds and pushes Docker images:

```yaml
docker:
  needs: build
  if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/')
  steps:
    - Build Docker image
    - Push to registry
```

**Required Secrets:**

| Secret            | Description                    |
| ----------------- | ------------------------------ |
| `DOCKER_USERNAME` | Docker registry username       |
| `DOCKER_PASSWORD` | Docker registry password/token |

### SonarQube Analysis (Optional)

When enabled, code quality analysis runs after tests:

**Required Secrets:**

| Secret        | Description                    |
| ------------- | ------------------------------ |
| `SONAR_TOKEN` | SonarQube authentication token |

## GitLab CI Configuration

When you run `jhipster-rust ci-cd` and select GitLab CI, a configuration is generated at `.gitlab-ci.yml`:

### Pipeline Stages

```yaml
stages:
  - lint
  - build
  - test
  - package # Optional: Docker publishing
```

### Jobs

- **lint**: Runs `cargo clippy --all-targets -- -D warnings`
- **build**: Runs `cargo build --release`
- **test**: Runs `cargo test`
- **package**: Builds and pushes Docker image (optional)

## Skipping CI

To skip CI for a commit, include one of these in your commit message:

- `[ci skip]`
- `[skip ci]`

## Troubleshooting

### Tests fail with "duplicate key" errors (PostgreSQL/MySQL)

This happens when tests run in parallel and try to run migrations simultaneously.

**Solution:** The generated workflow uses `--test-threads=1` to prevent this. For local testing:

```bash
cargo test -- --test-threads=1
```

### rdkafka build fails with "cmake not found"

The `rdkafka` crate requires `cmake` to build `librdkafka`.

**Solution:** The generated workflow automatically installs cmake for Kafka projects. For local development:

```bash
# macOS
brew install cmake

# Linux
sudo apt-get install cmake
```

### Clippy warnings fail the build

The pipeline uses `cargo clippy -- -D warnings` which treats warnings as errors.

**Solution:** Fix linting issues locally:

```bash
cargo clippy --fix --allow-dirty
```

### Cache not working

Verify cache keys match your `Cargo.lock` structure. Clear cache if needed:

```bash
# GitHub Actions: Go to Settings > Actions > Caches and delete
# act: Remove ~/.cache/act/
```

### Slow builds on Apple Silicon with act

This is expected due to x86_64 emulation. Builds will be faster on actual GitHub Actions runners.

## Customization

After generation, you can customize the pipeline to:

- Add deployment stages
- Configure additional environments (staging, production)
- Add notifications (Slack, email, etc.)
- Include additional security scanning tools
- Add end-to-end testing with Cypress

## Related Documentation

- [Docker Integration](DOCKER.md)
- [Testing Guide](TESTING.md)
- [Security Guide](SECURITY.md)
