use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::db::connection::DbConnection;
use crate::errors::AppError;
use crate::services::UserService;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub auth: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

pub struct AuthService;

impl AuthService {
    /// Authenticate user and return JWT token
    pub fn authenticate(
        conn: &mut DbConnection,
        config: &AppConfig,
        username: &str,
        password: &str,
        remember_me: bool,
    ) -> Result<String, AppError> {
        // Find user by login
        let user = UserService::find_by_login(conn, username).map_err(|_| {
            AppError::Unauthorized("Invalid username or password".to_string())
        })?;

        // Check if user is activated
        if !user.activated {
            return Err(AppError::Unauthorized("User is not activated".to_string()));
        }

        // Verify password
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| AppError::Internal("Invalid password hash".to_string()))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Invalid username or password".to_string()))?;

        // Get user authorities
        let authorities = UserService::get_authorities(conn, user.id)?;

        // Generate JWT token
        let token = Self::generate_token(config, &user.login, &authorities, remember_me)?;

        Ok(token)
    }

    /// Generate JWT token
    pub fn generate_token(
        config: &AppConfig,
        login: &str,
        authorities: &[String],
        remember_me: bool,
    ) -> Result<String, AppError> {
        let now = Utc::now();
        let expiration = if remember_me {
            now + Duration::days(30)
        } else {
            now + Duration::hours(config.jwt_expiration_hours as i64)
        };

        let claims = Claims {
            sub: login.to_string(),
            auth: authorities.to_vec(),
            iat: now.timestamp() as usize,
            exp: expiration.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))
    }

    /// Change user password
    pub fn change_password(
        conn: &mut DbConnection,
        login: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        // Find user
        let user = UserService::find_by_login(conn, login)?;

        // Verify current password
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| AppError::Internal("Invalid password hash".to_string()))?;

        Argon2::default()
            .verify_password(current_password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::BadRequest("Current password is incorrect".to_string()))?;

        // Hash new password
        let new_hash = UserService::hash_password(new_password)?;

        // Update password
        UserService::update_password(conn, login, &new_hash)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn test_config() -> AppConfig {
        AppConfig {
            app_name: "test".to_string(),
            app_env: "test".to_string(),
            app_port: 8080,
            app_host: "127.0.0.1".to_string(),
            app_https: false,
            database_url: "test.db".to_string(),
            jwt_secret: "test_secret_key_for_jwt_token_generation_12345".to_string(),
            jwt_expiration_hours: 24,
            static_files_dir: None,
            serve_static_files: false,
        }
    }

    #[test]
    fn test_generate_token_success() {
        let config = test_config();
        let result = AuthService::generate_token(&config, "testuser", &["ROLE_USER".to_string()], false);
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_generate_token_with_remember_me() {
        let config = test_config();
        let result = AuthService::generate_token(&config, "testuser", &["ROLE_USER".to_string()], true);
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_generate_token_with_multiple_authorities() {
        let config = test_config();
        let authorities = vec!["ROLE_USER".to_string(), "ROLE_ADMIN".to_string()];
        let result = AuthService::generate_token(&config, "admin", &authorities, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_claims_serialization() {
        let claims = Claims {
            sub: "testuser".to_string(),
            auth: vec!["ROLE_USER".to_string()],
            exp: 1234567890,
            iat: 1234567800,
        };
        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("\"sub\":\"testuser\""));
        assert!(json.contains("\"auth\":[\"ROLE_USER\"]"));
        assert!(json.contains("\"exp\":1234567890"));
        assert!(json.contains("\"iat\":1234567800"));
    }

    #[test]
    fn test_claims_deserialization() {
        let json = r#"{"sub":"testuser","auth":["ROLE_USER","ROLE_ADMIN"],"exp":1234567890,"iat":1234567800}"#;
        let claims: Claims = serde_json::from_str(json).unwrap();
        assert_eq!(claims.sub, "testuser");
        assert_eq!(claims.auth.len(), 2);
        assert_eq!(claims.exp, 1234567890);
        assert_eq!(claims.iat, 1234567800);
    }

    #[test]
    fn test_token_contains_valid_jwt_structure() {
        let config = test_config();
        let token = AuthService::generate_token(&config, "user", &["ROLE_USER".to_string()], false).unwrap();
        // JWT tokens have 3 parts separated by dots
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have 3 parts");
    }

    #[test]
    fn test_claims_debug() {
        let claims = Claims {
            sub: "testuser".to_string(),
            auth: vec!["ROLE_USER".to_string()],
            exp: 1234567890,
            iat: 1234567800,
        };
        let debug_str = format!("{:?}", claims);
        assert!(debug_str.contains("testuser"));
        assert!(debug_str.contains("ROLE_USER"));
    }

    #[test]
    fn test_generate_token_empty_authorities() {
        let config = test_config();
        let result = AuthService::generate_token(&config, "testuser", &[], false);
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
    }
}

/// Integration tests that require database access
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_utils::{create_test_pool, create_test_config, create_test_admin, create_test_user};

    #[test]
    fn test_authenticate_success() {
        let pool = create_test_pool();
        let config = create_test_config();
        let mut conn = pool.get().unwrap();

        // Create admin user with password "admin123"
        create_test_admin(&pool);

        // Authenticate
        let result = AuthService::authenticate(&mut conn, &config, "admin", "admin123", false);
        assert!(result.is_ok());

        let token = result.unwrap();
        assert!(!token.is_empty());
        // Verify it's a valid JWT structure
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_authenticate_with_remember_me() {
        let pool = create_test_pool();
        let config = create_test_config();
        let mut conn = pool.get().unwrap();

        create_test_admin(&pool);

        let result = AuthService::authenticate(&mut conn, &config, "admin", "admin123", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_authenticate_wrong_password() {
        let pool = create_test_pool();
        let config = create_test_config();
        let mut conn = pool.get().unwrap();

        create_test_admin(&pool);

        let result = AuthService::authenticate(&mut conn, &config, "admin", "wrongpassword", false);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(msg) => assert!(msg.contains("Invalid")),
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_authenticate_user_not_found() {
        let pool = create_test_pool();
        let config = create_test_config();
        let mut conn = pool.get().unwrap();

        let result = AuthService::authenticate(&mut conn, &config, "nonexistent", "password", false);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(_) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_authenticate_deactivated_user() {
        let pool = create_test_pool();
        let config = create_test_config();
        let mut conn = pool.get().unwrap();

        // Create a user and then deactivate them
        create_test_user(&pool);
        use crate::dto::UpdateUserDto;
        let user = UserService::find_by_login(&mut conn, "user").unwrap();
        UserService::update(&mut conn, user.id, UpdateUserDto {
            login: None,
            first_name: None,
            last_name: None,
            email: None,
            activated: Some(false),
            lang_key: None,
            image_url: None,
            authorities: None,
        }, "system").unwrap();

        let result = AuthService::authenticate(&mut conn, &config, "user", "user123", false);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(msg) => assert!(msg.contains("not activated")),
            _ => panic!("Expected Unauthorized error about activation"),
        }
    }

    #[test]
    fn test_change_password_success() {
        let pool = create_test_pool();
        let config = create_test_config();
        let mut conn = pool.get().unwrap();

        create_test_admin(&pool);

        // Change password
        let result = AuthService::change_password(&mut conn, "admin", "admin123", "newpassword456");
        assert!(result.is_ok());

        // Verify old password no longer works
        let auth_result = AuthService::authenticate(&mut conn, &config, "admin", "admin123", false);
        assert!(auth_result.is_err());

        // Verify new password works
        let auth_result = AuthService::authenticate(&mut conn, &config, "admin", "newpassword456", false);
        assert!(auth_result.is_ok());
    }

    #[test]
    fn test_change_password_wrong_current() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        create_test_admin(&pool);

        let result = AuthService::change_password(&mut conn, "admin", "wrongpassword", "newpassword456");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::BadRequest(msg) => assert!(msg.contains("incorrect")),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_change_password_user_not_found() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let result = AuthService::change_password(&mut conn, "nonexistent", "oldpass", "newpass");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(_) => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_authenticate_regular_user() {
        let pool = create_test_pool();
        let config = create_test_config();
        let mut conn = pool.get().unwrap();

        // Create regular user with password "user123"
        create_test_user(&pool);

        let result = AuthService::authenticate(&mut conn, &config, "user", "user123", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_token_contains_correct_authorities() {
        let pool = create_test_pool();
        let config = create_test_config();
        let mut conn = pool.get().unwrap();

        create_test_admin(&pool);

        let token = AuthService::authenticate(&mut conn, &config, "admin", "admin123", false).unwrap();

        // Decode and verify claims (basic check - token is valid JWT)
        use jsonwebtoken::{decode, DecodingKey, Validation};
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &Validation::default(),
        ).unwrap();

        assert_eq!(token_data.claims.sub, "admin");
        assert!(token_data.claims.auth.contains(&"ROLE_ADMIN".to_string()));
        assert!(token_data.claims.auth.contains(&"ROLE_USER".to_string()));
    }
}
