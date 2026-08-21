//! Test utilities for integration testing
//!
//! Provides helpers for setting up test PostgreSQL database and test fixtures.

use std::sync::OnceLock;
use diesel::r2d2::{self, ConnectionManager};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_migrations::MigrationHarness;

use crate::config::AppConfig;
use crate::db::connection::{DbPool, MIGRATIONS};
use crate::models::NewUser;
use crate::services::UserService;
use crate::AppState;

/// Stores the shared pool once migrations succeed. Using OnceLock instead of
/// Once avoids poisoning — if the database isn't reachable on the first attempt
/// each test will retry independently instead of cascading failures.
static POOL: OnceLock<DbPool> = OnceLock::new();

fn test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .or_else(|_| {
            std::env::var("DATABASE_URL").map(|base_url| {
                format!("{}_test", base_url)
            })
        })
        .unwrap_or_else(|_| "postgres://jobSearchRust@localhost:5432/jobSearchRust_test".to_string())
}

/// Creates a test database pool connected to a test database.
///
/// Uses the TEST_DATABASE_URL environment variable if set,
/// otherwise falls back to DATABASE_URL with "_test" appended,
/// or uses a default local PostgreSQL connection.
pub fn create_test_pool() -> DbPool {
    let pool = POOL.get_or_init(|| {
        let database_url = test_database_url();
        ensure_test_database_exists(&database_url);

        let manager = ConnectionManager::<PgConnection>::new(&database_url);
        let pool = r2d2::Pool::builder()
            .max_size(5)
            .build(manager)
            .expect("Failed to create test pool. Ensure PostgreSQL is running and accessible.");

        let mut conn = pool.get().expect("Failed to get connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        pool
    });

    // Clean up existing test data for isolation
    let mut conn = pool.get().expect("Failed to get connection");
    clean_test_data(&mut conn);

    pool.clone()
}

/// Ensures the test database exists
fn ensure_test_database_exists(database_url: &str) {
    let db_name = database_url.rsplit('/').next().unwrap_or("test");
    let maintenance_url = if let Some(last_slash) = database_url.rfind('/') {
        format!("{}postgres", &database_url[..=last_slash])
    } else {
        database_url.to_string()
    };

    if let Ok(mut conn) = PgConnection::establish(&maintenance_url) {
        // Check if database exists
        let db_exists: Result<Option<CountResult>, _> = diesel::sql_query(
            format!("SELECT 1 as count FROM pg_database WHERE datname = '{}'", db_name)
        )
        .get_result::<CountResult>(&mut conn)
        .optional();

        if let Ok(None) = db_exists {
            let _ = diesel::sql_query(format!("CREATE DATABASE \"{}\"", db_name))
                .execute(&mut conn);
        }
    }
}

#[derive(diesel::QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    #[allow(dead_code)]
    count: i64,
}

/// Cleans up test data for isolation between tests
fn clean_test_data(conn: &mut PgConnection) {
    use crate::db::schema::{user_authorities, users};

    // Delete test users (keep admin and user from migrations)
    let _ = diesel::delete(user_authorities::table.filter(
        user_authorities::user_id.ne_all(
            users::table.select(users::id).filter(
                users::login.eq("admin").or(users::login.eq("user"))
            )
        )
    )).execute(conn);

    let _ = diesel::delete(users::table.filter(
        users::login.ne("admin").and(users::login.ne("user"))
    )).execute(conn);
}

/// Creates a test configuration
pub fn create_test_config() -> AppConfig {
    AppConfig {
        app_name: "test_app".to_string(),
        app_env: "test".to_string(),
        app_port: 8080,
        app_host: "127.0.0.1".to_string(),
        app_https: false,
        database_url: std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://jobSearchRust@localhost:5432/jobSearchRust_test".to_string()),
        jwt_secret: "test_jwt_secret_key_for_testing_purposes_only_12345".to_string(),
        jwt_expiration_hours: 24,
        static_files_dir: None,
        serve_static_files: false,
    }
}

/// Creates a test AppState with test database
pub fn create_test_state() -> AppState {
    AppState {
        pool: create_test_pool(),
        config: create_test_config(),
    }
}

/// Gets or creates a test admin user and returns their login
///
/// If the admin user already exists (from migrations), updates the password to "admin123".
/// Otherwise creates a new admin user with password "admin123".
pub fn create_test_admin(pool: &DbPool) -> String {
    let mut conn = pool.get().expect("Failed to get connection");

    // Check if admin already exists (from migrations)
    if let Ok(_user) = UserService::find_by_login(&mut conn, "admin") {
        // Admin exists - update password to known test value
        let password_hash = UserService::hash_password("admin123").expect("Failed to hash password");
        UserService::update_password(&mut conn, "admin", &password_hash)
            .expect("Failed to update admin password");
        return "admin".to_string();
    }

    // Admin doesn't exist - create new one
    let password_hash = UserService::hash_password("admin123").expect("Failed to hash password");

    let new_user = NewUser {
        login: "admin".to_string(),
        password_hash,
        first_name: Some("Admin".to_string()),
        last_name: Some("User".to_string()),
        email: "admin@localhost".to_string(),
        activated: true,
        lang_key: Some("en".to_string()),
        image_url: None,
        created_by: Some("system".to_string()),
        created_date: Some(chrono::Utc::now().naive_utc()),
        last_modified_by: Some("system".to_string()),
        last_modified_date: Some(chrono::Utc::now().naive_utc()),
    };

    UserService::create_with_authorities(
        &mut conn,
        new_user,
        vec!["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
    )
    .expect("Failed to create admin user");

    "admin".to_string()
}

/// Gets or creates a test regular user and returns their login
///
/// If the user already exists (from migrations), updates the password to "user123"
/// and ensures the user is activated. Otherwise creates a new user.
pub fn create_test_user(pool: &DbPool) -> String {
    let mut conn = pool.get().expect("Failed to get connection");

    // Check if user already exists (from migrations)
    if let Ok(user) = UserService::find_by_login(&mut conn, "user") {
        // User exists - update password to known test value and ensure activated
        let password_hash = UserService::hash_password("user123").expect("Failed to hash password");
        UserService::update_password(&mut conn, "user", &password_hash)
            .expect("Failed to update user password");
        // Ensure user is activated (may have been deactivated by a previous test)
        use crate::dto::UpdateUserDto;
        UserService::update(&mut conn, user.id, UpdateUserDto {
            login: None,
            first_name: None,
            last_name: None,
            email: None,
            activated: Some(true),
            lang_key: None,
            image_url: None,
            authorities: None,
        }, "system").expect("Failed to reactivate user");
        return "user".to_string();
    }

    // User doesn't exist - create new one
    let password_hash = UserService::hash_password("user123").expect("Failed to hash password");

    let new_user = NewUser {
        login: "user".to_string(),
        password_hash,
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        email: "user@localhost".to_string(),
        activated: true,
        lang_key: Some("en".to_string()),
        image_url: None,
        created_by: Some("system".to_string()),
        created_date: Some(chrono::Utc::now().naive_utc()),
        last_modified_by: Some("system".to_string()),
        last_modified_date: Some(chrono::Utc::now().naive_utc()),
    };

    UserService::create_with_authorities(
        &mut conn,
        new_user,
        vec!["ROLE_USER".to_string()],
    )
    .expect("Failed to create test user");

    "user".to_string()
}

/// Generates a valid JWT token for testing
pub fn generate_test_token(config: &AppConfig, login: &str, authorities: &[String]) -> String {
    crate::services::AuthService::generate_token(config, login, authorities, false)
        .expect("Failed to generate test token")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_pool() {
        let pool = create_test_pool();
        assert!(pool.get().is_ok());
    }

    #[test]
    fn test_create_test_config() {
        let config = create_test_config();
        assert_eq!(config.app_env, "test");
        assert!(!config.jwt_secret.is_empty());
    }

    #[test]
    fn test_create_test_state() {
        let state = create_test_state();
        assert!(state.pool.get().is_ok());
        assert_eq!(state.config.app_env, "test");
    }

    #[test]
    fn test_create_test_admin() {
        let pool = create_test_pool();
        let login = create_test_admin(&pool);
        assert_eq!(login, "admin");

        // Verify user exists
        let mut conn = pool.get().unwrap();
        let user = UserService::find_by_login(&mut conn, "admin").unwrap();
        assert_eq!(user.email, "admin@localhost");
        assert!(user.activated);
    }

    #[test]
    fn test_create_test_user() {
        let pool = create_test_pool();
        let login = create_test_user(&pool);
        assert_eq!(login, "user");

        // Verify user exists
        let mut conn = pool.get().unwrap();
        let user = UserService::find_by_login(&mut conn, "user").unwrap();
        assert_eq!(user.email, "user@localhost");
    }

    #[test]
    fn test_generate_test_token() {
        let config = create_test_config();
        let token = generate_test_token(&config, "testuser", &["ROLE_USER".to_string()]);
        assert!(!token.is_empty());
        // JWT tokens have 3 parts
        assert_eq!(token.split('.').count(), 3);
    }
}
