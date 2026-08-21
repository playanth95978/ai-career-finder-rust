# Testing Guide

This guide covers testing strategies for JHipster Rust applications, including unit tests, integration tests, and end-to-end (E2E) tests with Cypress.

## Overview

JHipster Rust generates a comprehensive test suite covering:

| Test Type         | Framework        | Location             | Purpose                           |
| ----------------- | ---------------- | -------------------- | --------------------------------- |
| Unit Tests        | Rust built-in    | `server/src/**/*.rs` | Test individual functions/modules |
| Integration Tests | Rust + axum-test | `server/src/**/*.rs` | Test API endpoints with database  |
| E2E Tests         | Cypress          | `client/cypress/`    | Test full user workflows          |

## Running Tests

### Quick Reference

```bash
# Run all Rust tests
cd server && cargo test

# Run Rust tests with output
cd server && cargo test -- --nocapture

# Run specific test
cd server && cargo test test_name

# Run E2E tests (headless)
cd client && npm run e2e:headless

# Run E2E tests (interactive)
cd client && npm run e2e
```

## Unit Tests

Unit tests are embedded within the source files using Rust's built-in `#[cfg(test)]` module pattern.

### Structure

```rust
// src/services/user_service.rs

pub struct UserService;

impl UserService {
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        // Implementation...
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let password = "my_secure_password";
        let result = UserService::hash_password(password);
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_different_outputs() {
        let password = "same_password";
        let hash1 = UserService::hash_password(password).unwrap();
        let hash2 = UserService::hash_password(password).unwrap();
        // Due to random salt, same password produces different hashes
        assert_ne!(hash1, hash2);
    }
}
```

### Common Unit Test Patterns

#### Testing Serialization/Deserialization

```rust
#[test]
fn test_login_request_deserialization() {
    let json = r#"{"username":"testuser","password":"password123","remember_me":true}"#;
    let request: LoginRequest = serde_json::from_str(json).unwrap();
    assert_eq!(request.username, "testuser");
    assert_eq!(request.password, "password123");
    assert!(request.remember_me);
}

#[test]
fn test_login_response_serialization() {
    let response = LoginResponse {
        id_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.signature".to_string(),
    };
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("id_token"));
}
```

#### Testing Default Values

```rust
#[test]
fn test_login_request_default_remember_me() {
    let json = r#"{"username":"testuser","password":"password123"}"#;
    let request: LoginRequest = serde_json::from_str(json).unwrap();
    assert!(!request.remember_me); // Defaults to false
}
```

## Integration Tests

Integration tests use the `test_utils` module to create isolated test databases and verify API behavior.

### Test Utilities

The generated `test_utils.rs` provides helper functions:

| Function                | Description                                        |
| ----------------------- | -------------------------------------------------- |
| `create_test_pool()`    | Creates isolated test database connection pool     |
| `create_test_config()`  | Creates test configuration                         |
| `create_test_state()`   | Creates full AppState for handler tests            |
| `create_test_admin()`   | Creates/retrieves admin user (password: admin123)  |
| `create_test_user()`    | Creates/retrieves regular user (password: user123) |
| `generate_test_token()` | Generates JWT token for authenticated requests     |

### Database-Specific Setup

#### SQLite (Default)

SQLite tests use unique temporary database files for isolation:

```rust
pub fn create_test_pool() -> DbPool {
    // Creates unique temp file: /tmp/test_db_<id>_<thread>_<timestamp>.sqlite
    let db_path = std::env::temp_dir().join(format!(
        "test_db_{}_{}_{}.sqlite",
        db_id, thread_id, timestamp
    ));
    // ... pool creation and migrations
}
```

#### PostgreSQL

PostgreSQL tests use a separate test database:

```rust
// Uses TEST_DATABASE_URL or appends "_test" to DATABASE_URL
let database_url = std::env::var("TEST_DATABASE_URL")
    .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/myapp_test".to_string());
```

#### MongoDB

MongoDB tests use a separate test database:

```rust
// Uses TEST_MONGODB_URI or MONGODB_URI
let mongodb_uri = std::env::var("TEST_MONGODB_URI")
    .or_else(|_| std::env::var("MONGODB_URI"))
    .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

// Uses TEST_MONGODB_DATABASE or appends "_test" to MONGODB_DATABASE
let database_name = std::env::var("TEST_MONGODB_DATABASE")
    .or_else(|_| std::env::var("MONGODB_DATABASE").map(|s| format!("{}_test", s)))
    .unwrap_or_else(|_| "myapp_test".to_string());
```

### Writing Integration Tests

#### Service Layer Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_utils::create_test_pool;
    use crate::dto::{CreateUserDto, PageRequest};

    #[test]
    fn test_create_user() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let dto = CreateUserDto {
            login: "testuser".to_string(),
            password: "password123".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            email: "testuser@example.com".to_string(),
            activated: Some(true),
            lang_key: Some("en".to_string()),
            image_url: None,
            authorities: Some(vec!["ROLE_USER".to_string()]),
        };

        let result = UserService::create(&mut conn, dto, "system");
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.login, "testuser");
        assert_eq!(user.email, "testuser@example.com");
        assert!(user.activated);
    }

    #[test]
    fn test_find_all_with_pagination() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let page_request = PageRequest {
            page: Some(0),
            size: Some(10),
            sort: vec!["login,asc".to_string()],
        };

        let (users, total) = UserService::find_all(&mut conn, &page_request).unwrap();
        assert!(total >= 0);
    }
}
```

#### Handler Tests with HTTP Client

For full API tests, use `axum-test`:

```rust
#[cfg(test)]
mod api_tests {
    use axum_test::TestServer;
    use crate::test_utils::{create_test_state, create_test_admin, generate_test_token};

    #[tokio::test]
    async fn test_get_account() {
        let state = create_test_state();
        let app = create_app(state.clone());
        let server = TestServer::new(app).unwrap();

        // Create admin and get token
        create_test_admin(&state.pool);
        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()]
        );

        let response = server
            .get("/api/account")
            .add_header("Authorization", format!("Bearer {}", token))
            .await;

        response.assert_status_ok();
    }
}
```

### Running Integration Tests

```bash
# Run all tests
cargo test

# Run only integration tests (by module name pattern)
cargo test integration

# Run with database logging
RUST_LOG=diesel=debug cargo test

# Run tests sequentially (useful for shared database)
cargo test -- --test-threads=1
```

### Environment Variables for Tests

| Variable                | Description                        | Example                                     |
| ----------------------- | ---------------------------------- | ------------------------------------------- |
| `TEST_DATABASE_URL`     | PostgreSQL/MySQL test database URL | `postgres://user:pass@localhost/myapp_test` |
| `TEST_MONGODB_URI`      | MongoDB test connection URI        | `mongodb://localhost:27017`                 |
| `TEST_MONGODB_DATABASE` | MongoDB test database name         | `myapp_test`                                |

## E2E Tests with Cypress

Cypress provides end-to-end testing for the full application stack (frontend + backend).

### Prerequisites

1. Start the database (Docker):

   ```bash
   docker compose up -d
   ```

2. Start the Rust backend:

   ```bash
   cd server && cargo run
   ```

3. Start the frontend dev server:
   ```bash
   cd client && npm start
   ```

### Running Cypress Tests

#### Interactive Mode

Opens Cypress Test Runner with visual feedback:

```bash
cd client
npm run e2e
```

#### Headless Mode

Runs tests in CI/CD environments:

```bash
cd client
npm run e2e:headless
```

#### Wait for Backend

Use the wait script to ensure the backend is ready:

```bash
cd client
npm run e2e:wait-backend && npm run e2e:headless
```

### Test Structure

Cypress tests are organized by feature:

```
client/cypress/
├── e2e/
│   ├── account/
│   │   ├── login-page.cy.ts        # Login functionality
│   │   ├── register-page.cy.ts     # User registration
│   │   ├── reset-password-page.cy.ts # Password reset
│   │   └── settings-page.cy.ts     # Account settings
│   ├── admin/
│   │   └── user-management.cy.ts   # Admin user management
│   └── entity/
│       └── *.cy.ts                 # Entity CRUD tests
├── support/
│   ├── commands.ts                 # Custom Cypress commands
│   └── entity.ts                   # Entity test helpers
└── fixtures/
    └── *.json                      # Test data
```

### Custom Commands

JHipster provides custom Cypress commands for common operations:

```typescript
// Login as admin
cy.login('admin', 'admin');

// Login as user
cy.login('user', 'user');

// Logout
cy.logout();

// Navigate to entity list
cy.clickOnEntityMenuItem('myEntity');
```

### OAuth2 Considerations

When using OAuth2 authentication:

- Login tests are skipped by default (require Keycloak setup)
- Use `describe.skip` for OAuth2-specific test suites
- Configure Keycloak test users for full E2E testing

### MongoDB Considerations

MongoDB uses ObjectId instead of numeric IDs:

- Entity tests may show ObjectId format (24-character hex strings)
- Relationships reference ObjectIds instead of integers
- Sorting by `_id` uses lexicographic ordering

### Email Disabled Behavior

When email is disabled (`MAIL_ENABLED=false`):

- Password reset tests are automatically skipped
- Registration creates already-activated users
- Activation tests are not applicable

## Test Coverage

### Generating Coverage Reports

Use `cargo-tarpaulin` for code coverage:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

### Coverage Configuration

Create `tarpaulin.toml` for custom settings:

```toml
[tarpaulin]
ignore-tests = true
out = ["Html", "Lcov"]
exclude-files = ["src/main.rs", "src/test_utils.rs"]
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: myapp_test
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run Rust tests
        working-directory: server
        env:
          TEST_DATABASE_URL: postgres://postgres:postgres@localhost:5432/myapp_test
        run: cargo test

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Start services
        run: docker compose up -d

      - name: Build and start backend
        working-directory: server
        run: |
          cargo build --release
          ./target/release/myapp &

      - name: Run Cypress tests
        working-directory: client
        run: |
          npm ci
          npm run e2e:wait-backend
          npm run e2e:headless
```

## Best Practices

### Test Isolation

1. **Use unique databases per test** (SQLite temp files)
2. **Clean up test data** between tests
3. **Don't rely on test execution order**
4. **Use `--test-threads=1`** for shared database tests

### Naming Conventions

```rust
#[test]
fn test_<function_name>_<scenario>() {
    // test_create_user_success
    // test_create_user_duplicate_email
    // test_find_by_id_not_found
}
```

### Test Organization

```rust
#[cfg(test)]
mod tests {
    mod unit_tests {
        // Pure function tests, no database
    }

    mod integration_tests {
        // Database and service tests
    }
}
```

### Async Tests (MongoDB)

```rust
#[tokio::test]
async fn test_mongodb_operation() {
    let db = create_test_mongo_pool().await;
    // async operations...
}
```

### Assertions

```rust
// Prefer specific assertions
assert_eq!(user.login, "admin");
assert!(user.activated);
assert!(result.is_ok());
assert!(result.is_err());

// Check error types
assert!(matches!(result, Err(AppError::NotFound(_))));
```

## Troubleshooting

### Database Connection Issues

```bash
# Check if database is running
docker compose ps

# Check connection string
echo $DATABASE_URL
echo $TEST_DATABASE_URL
```

### Test Isolation Failures

```bash
# Run tests sequentially
cargo test -- --test-threads=1

# Run specific failing test
cargo test test_name -- --nocapture
```

### Cypress Timeout Issues

```bash
# Increase wait timeout
npm run e2e:wait-backend  # waits 120 seconds by default

# Check backend health
curl http://localhost:8080/api/health
```

### OAuth2 Test Failures

For OAuth2 applications, ensure Keycloak is running and configured:

```bash
# Start Keycloak
docker compose -f docker-compose.yml up -d keycloak

# Check Keycloak health
curl http://localhost:9080/health
```
