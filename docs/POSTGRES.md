# PostgreSQL Integration

This document covers setting up and running PostgreSQL with the JHipster Rust generator.

## Prerequisites

### macOS Setup

On macOS, you need to install the PostgreSQL client library (`libpq`) for the Diesel ORM to compile:

```bash
brew install libpq
```

After installation, you need to configure the environment so that Rust can find the library during compilation.

#### Option 1: Force Link libpq (Recommended)

The simplest solution is to force-link libpq so it's available system-wide:

```bash
brew link --force libpq
```

This adds libpq to your PATH and makes it discoverable by the linker without additional configuration.

#### Option 2: Set Environment Variables

If you prefer not to force-link, you need to set **both** `PQ_LIB_DIR` and `LIBRARY_PATH` environment variables. Setting only `PQ_LIB_DIR` is not sufficient for the linker to find the library at runtime.

**Apple Silicon (M1/M2/M3):**

```bash
export PQ_LIB_DIR="/opt/homebrew/opt/libpq/lib"
export LIBRARY_PATH="/opt/homebrew/opt/libpq/lib:$LIBRARY_PATH"
```

**Intel Mac:**

```bash
export PQ_LIB_DIR="/usr/local/opt/libpq/lib"
export LIBRARY_PATH="/usr/local/opt/libpq/lib:$LIBRARY_PATH"
```

You can add these to your shell profile (`~/.zshrc` or `~/.bashrc`) to make them permanent:

```bash
# For Apple Silicon
echo 'export PQ_LIB_DIR="/opt/homebrew/opt/libpq/lib"' >> ~/.zshrc
echo 'export LIBRARY_PATH="/opt/homebrew/opt/libpq/lib:$LIBRARY_PATH"' >> ~/.zshrc

# For Intel Mac
echo 'export PQ_LIB_DIR="/usr/local/opt/libpq/lib"' >> ~/.zshrc
echo 'export LIBRARY_PATH="/usr/local/opt/libpq/lib:$LIBRARY_PATH"' >> ~/.zshrc
```

After updating your shell profile, run `source ~/.zshrc` or open a new terminal window.

#### Alternative: Using .cargo/config.toml

You can also create a `.cargo/config.toml` file in your server directory:

```toml
[env]
PQ_LIB_DIR = "/opt/homebrew/opt/libpq/lib"

[target.'cfg(target_os = "macos")']
rustflags = ["-L", "/opt/homebrew/opt/libpq/lib", "-L", "/usr/local/opt/libpq/lib"]
```

### Linux Setup

On most Linux distributions, install the PostgreSQL development package:

**Ubuntu/Debian:**

```bash
sudo apt-get install libpq-dev
```

**Fedora/RHEL:**

```bash
sudo dnf install postgresql-devel
```

**Arch Linux:**

```bash
sudo pacman -S postgresql-libs
```

### Windows Setup

On Windows, you can install PostgreSQL and add the `bin` directory to your PATH, or set `PQ_LIB_DIR` to point to the PostgreSQL `lib` directory.

## Running PostgreSQL with Docker

The generated project includes a Docker Compose file for running PostgreSQL.

### Starting the Database

```bash
docker compose -f docker/postgresql.yml up -d
```

### Stopping the Database

```bash
docker compose -f docker/postgresql.yml down
```

### Viewing Logs

```bash
docker compose -f docker/postgresql.yml logs -f
```

### Default Configuration

| Setting  | Value                                |
| -------- | ------------------------------------ |
| Host     | localhost                            |
| Port     | 5432                                 |
| Database | `<appname>` (based on your app name) |
| Username | `<appname>` (based on your app name) |
| Password | (empty or as configured in .env)     |

The database URL is configured in the `.env` file:

```
DATABASE_URL=postgres://<appname>:@localhost:5432/<appname>
```

## Running Migrations

Before starting the server, run the database migrations:

```bash
cd server
diesel migration run
```

To revert the last migration:

```bash
diesel migration revert
```

To check migration status:

```bash
diesel migration pending
```

## Running the Server

Once PostgreSQL is running and migrations are applied:

```bash
cd server
cargo run
```

The server will start on `http://localhost:8080`.

## Running Tests

Integration tests require a running PostgreSQL database. The tests will automatically create a separate test database (appending `_test` to the database name).

```bash
# Run all tests (with single thread for database tests)
cargo test -- --test-threads=1

# Run only unit tests (no database required)
cargo test --lib -- --skip integration
```

### Test Database Configuration

By default, tests use `DATABASE_URL` with `_test` appended to the database name. You can override this with:

```bash
export TEST_DATABASE_URL="postgres://user:password@localhost:5432/myapp_test"
cargo test -- --test-threads=1
```

## Troubleshooting

### Build Error: library 'pq' not found

```
error: linking with `cc` failed: exit status: 1
ld: library 'pq' not found
```

**Solution (macOS):** This error occurs because the linker cannot find the libpq library. You have two options:

1. **Force link libpq (recommended):**

   ```bash
   brew link --force libpq
   ```

2. **Set both environment variables:**

   ```bash
   # Apple Silicon
   export PQ_LIB_DIR="/opt/homebrew/opt/libpq/lib"
   export LIBRARY_PATH="/opt/homebrew/opt/libpq/lib:$LIBRARY_PATH"

   # Intel Mac
   export PQ_LIB_DIR="/usr/local/opt/libpq/lib"
   export LIBRARY_PATH="/usr/local/opt/libpq/lib:$LIBRARY_PATH"
   ```

   **Note:** Setting only `PQ_LIB_DIR` is not sufficient. You must also set `LIBRARY_PATH` for the linker to find the library.

See the [macOS Setup](#macos-setup) section for more details.

### Connection Refused

```
connection to server at "localhost" (::1), port 5432 failed: Connection refused
```

**Solution:** Ensure PostgreSQL is running:

```bash
docker compose -f docker/postgresql.yml up -d
```

### Authentication Failed

```
FATAL: password authentication failed for user "postgres"
```

**Solution:** Check your `.env` file and ensure the `DATABASE_URL` matches your PostgreSQL configuration.

### Permission Denied Creating Database

If the test utilities fail to create the test database:

```bash
# Connect to PostgreSQL and create it manually
docker exec -it <container_name> psql -U postgres
CREATE DATABASE myapp_test;
\q
```

## Production Considerations

For production deployments:

1. **Use strong passwords**: Update the database password in your production `.env` file
2. **Enable SSL**: Configure SSL connections to PostgreSQL
3. **Connection pooling**: The application uses r2d2 for connection pooling with sensible defaults
4. **Backups**: Set up regular database backups
5. **Monitoring**: Monitor database performance and connection counts
