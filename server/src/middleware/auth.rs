use std::future::Future;
use std::pin::Pin;

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::services::Claims;
use crate::errors::AppError;
use crate::AppState;

/// Type alias for role middleware future to reduce type complexity
type RoleMiddlewareFuture = Pin<Box<dyn Future<Output = Result<Response, (StatusCode, String)>> + Send>>;

/// Authenticated user information extracted from JWT
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub login: String,
    pub authorities: Vec<String>,
}

impl AuthUser {
    pub fn has_authority(&self, authority: &str) -> bool {
        self.authorities.iter().any(|a| a == authority)
    }

    /// Creates an anonymous user (unauthenticated)
    pub fn anonymous() -> Self {
        Self {
            login: "anonymous".to_string(),
            authorities: vec![],
        }
    }

    /// Check if this is an anonymous (unauthenticated) user
    pub fn is_anonymous(&self) -> bool {
        self.login == "anonymous"
    }
}


/// Authentication middleware layer
pub struct AuthLayer;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Get authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let auth_user = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = header.trim_start_matches("Bearer ");
            validate_token(&state.config.jwt_secret, token)?
        }
        _ => {
            // Allow unauthenticated access for public endpoints
            AuthUser::anonymous()
        }
    };

    // Insert auth user into request extensions
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

/// Validate JWT token and extract claims
fn validate_token(secret: &str, token: &str) -> Result<AuthUser, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(AuthUser {
        login: token_data.claims.sub,
        authorities: token_data.claims.auth,
    })
}

/// Require authentication middleware
pub async fn require_auth(
    request: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth_user = request
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    if auth_user.is_anonymous() {
        return Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()));
    }

    Ok(next.run(request).await)
}

/// Require specific role middleware
pub fn require_role(
    role: &'static str,
) -> impl Fn(Request<Body>, Next) -> RoleMiddlewareFuture + Clone {
    move |request: Request<Body>, next: Next| {
        Box::pin(async move {
            let auth_user = request
                .extensions()
                .get::<AuthUser>()
                .cloned()
                .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

            if !auth_user.has_authority(role) {
                return Err((StatusCode::FORBIDDEN, format!("Role {} required", role)));
            }

            Ok(next.run(request).await)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod auth_user_tests {
        use super::*;

        #[test]
        fn test_auth_user_has_authority_true() {
            let user = AuthUser {
                login: "testuser".to_string(),
                authorities: vec!["ROLE_USER".to_string(), "ROLE_ADMIN".to_string()],
            };
            assert!(user.has_authority("ROLE_USER"));
            assert!(user.has_authority("ROLE_ADMIN"));
        }

        #[test]
        fn test_auth_user_has_authority_false() {
            let user = AuthUser {
                login: "testuser".to_string(),
                authorities: vec!["ROLE_USER".to_string()],
            };
            assert!(!user.has_authority("ROLE_ADMIN"));
        }

        #[test]
        fn test_auth_user_anonymous() {
            let user = AuthUser::anonymous();
            assert!(user.is_anonymous());
            assert!(!user.has_authority("ROLE_USER"));
        }

        #[test]
        fn test_auth_user_clone() {
            let user = AuthUser {
                login: "testuser".to_string(),
                authorities: vec!["ROLE_USER".to_string()],
            };
            let cloned = user.clone();
            assert_eq!(cloned.login, "testuser");
            assert_eq!(cloned.authorities.len(), 1);
        }

        #[test]
        fn test_auth_user_debug() {
            let user = AuthUser {
                login: "testuser".to_string(),
                authorities: vec!["ROLE_USER".to_string()],
            };
            let debug_str = format!("{:?}", user);
            assert!(debug_str.contains("testuser"));
            assert!(debug_str.contains("ROLE_USER"));
        }
    }

    mod validate_token_tests {
        use super::*;
        use jsonwebtoken::{encode, EncodingKey, Header};
        use chrono::{Duration, Utc};

        fn create_test_token(secret: &str, login: &str, authorities: Vec<String>, expired: bool) -> String {
            let now = Utc::now();
            let exp = if expired {
                now - Duration::hours(1)
            } else {
                now + Duration::hours(24)
            };

            let claims = Claims {
                sub: login.to_string(),
                auth: authorities,
                iat: now.timestamp() as usize,
                exp: exp.timestamp() as usize,
            };

            encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret.as_bytes()),
            )
            .unwrap()
        }

        #[test]
        fn test_validate_token_success() {
            let secret = "test_secret_key_for_jwt_token";
            let token = create_test_token(secret, "testuser", vec!["ROLE_USER".to_string()], false);
            let result = validate_token(secret, &token);
            assert!(result.is_ok());
            let auth_user = result.unwrap();
            assert_eq!(auth_user.login, "testuser");
            assert_eq!(auth_user.authorities, vec!["ROLE_USER"]);
        }

        #[test]
        fn test_validate_token_invalid_secret() {
            let token = create_test_token("correct_secret", "testuser", vec!["ROLE_USER".to_string()], false);
            let result = validate_token("wrong_secret", &token);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_token_expired() {
            let secret = "test_secret";
            let token = create_test_token(secret, "testuser", vec!["ROLE_USER".to_string()], true);
            let result = validate_token(secret, &token);
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_token_malformed() {
            let result = validate_token("secret", "not.a.valid.token");
            assert!(result.is_err());
        }

        #[test]
        fn test_validate_token_multiple_authorities() {
            let secret = "test_secret";
            let authorities = vec!["ROLE_USER".to_string(), "ROLE_ADMIN".to_string()];
            let token = create_test_token(secret, "admin", authorities.clone(), false);
            let result = validate_token(secret, &token);
            assert!(result.is_ok());
            let auth_user = result.unwrap();
            assert_eq!(auth_user.authorities.len(), 2);
        }
    }

    // Track 1 Phase 1d (2026-05-11): integration tests for the three
    // middleware functions (auth_middleware, require_auth, require_role).
    // Tests mount each middleware in a minimal axum Router with a downstream
    // echo handler that surfaces what was inserted into request extensions,
    // then drive the router via axum_test::TestServer.
    //
    // Gated to non-MongoDB (matches the SQL test_utils path) and JWT
    // (the OAuth2 middleware variant uses a different validator that needs
    // a live JWKS endpoint to test; that lives in a later phase).
    mod middleware_integration_tests {
        use super::super::*;
        use axum::{
            http::{header::AUTHORIZATION, HeaderValue},
            middleware,
            routing::get,
            Extension, Router,
        };
        use axum_test::TestServer;
        use jsonwebtoken::{encode, EncodingKey, Header};
        use chrono::{Duration, Utc};
        use crate::services::Claims;
        use crate::test_utils::{create_test_state, create_test_config};

        /// Build an `Authorization: Bearer <token>` header. Centralized so
        /// every test uses the same construction (and so HeaderValue parse
        /// failures fail loudly during test compile).
        fn bearer(token: &str) -> HeaderValue {
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap()
        }

        /// Downstream handler that echoes back whatever AuthUser is in
        /// request extensions, so tests can assert what auth_middleware
        /// put there. Returns 500 if there's no AuthUser at all.
        async fn echo_handler(Extension(user): Extension<AuthUser>) -> String {
            format!("{}:{}", user.login, user.authorities.join(","))
        }

        fn create_test_app() -> TestServer {
            let state = create_test_state();
            let app = Router::new()
                .route("/echo", get(echo_handler))
                .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
                .with_state(state);
            TestServer::new(app).unwrap()
        }

        fn token_with(login: &str, authorities: Vec<String>, expired: bool) -> String {
            let secret = create_test_config().jwt_secret;
            let now = Utc::now();
            let exp = if expired { now - Duration::hours(1) } else { now + Duration::hours(24) };
            let claims = Claims {
                sub: login.to_string(),
                auth: authorities,
                iat: now.timestamp() as usize,
                exp: exp.timestamp() as usize,
            };
            encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap()
        }

        #[tokio::test]
        async fn test_auth_middleware_no_header_results_in_anonymous_user() {
            // Public endpoints must work without auth — the middleware
            // inserts an anonymous AuthUser rather than rejecting.
            let server = create_test_app();
            let response = server.get("/echo").await;
            response.assert_status_ok();
            assert_eq!(response.text(), "anonymous:");
        }

        #[tokio::test]
        async fn test_auth_middleware_valid_bearer_token_populates_user() {
            let server = create_test_app();
            let token = token_with("alice", vec!["ROLE_USER".to_string()], false);
            let response = server
                .get("/echo")
                .add_header(AUTHORIZATION, bearer(&token))
                .await;
            response.assert_status_ok();
            assert_eq!(response.text(), "alice:ROLE_USER");
        }

        #[tokio::test]
        async fn test_auth_middleware_expired_token_returns_unauthorized() {
            // Documented contract: an expired Bearer token returns 401
            // (not an anonymous fallback). The downstream handler is never
            // reached.
            let server = create_test_app();
            let token = token_with("alice", vec!["ROLE_USER".to_string()], true);
            let response = server
                .get("/echo")
                .add_header(AUTHORIZATION, bearer(&token))
                .expect_failure()
                .await;
            assert_eq!(response.status_code(), axum::http::StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn test_auth_middleware_invalid_token_returns_unauthorized() {
            let server = create_test_app();
            let response = server
                .get("/echo")
                .add_header(AUTHORIZATION, HeaderValue::from_static("Bearer not.a.valid.token"))
                .expect_failure()
                .await;
            assert_eq!(response.status_code(), axum::http::StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn test_auth_middleware_non_bearer_header_falls_through_to_anonymous() {
            // Authorization header present but missing "Bearer " prefix —
            // treated as no token, anonymous user.
            let server = create_test_app();
            let response = server
                .get("/echo")
                .add_header(AUTHORIZATION, HeaderValue::from_static("Basic dXNlcjpwYXNz"))
                .await;
            response.assert_status_ok();
            assert_eq!(response.text(), "anonymous:");
        }

        // require_auth tests: composes on top of auth_middleware. A
        // separate router demonstrates the "must authenticate" gate.
        async fn ok_handler() -> &'static str { "ok" }

        fn create_require_auth_app() -> TestServer {
            let state = create_test_state();
            let app = Router::new()
                .route("/protected", get(ok_handler))
                .layer(middleware::from_fn(require_auth))
                .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
                .with_state(state);
            TestServer::new(app).unwrap()
        }

        #[tokio::test]
        async fn test_require_auth_rejects_anonymous_user() {
            // No Authorization header → auth_middleware inserts anonymous,
            // require_auth rejects with 401.
            let server = create_require_auth_app();
            let response = server.get("/protected").expect_failure().await;
            assert_eq!(response.status_code(), axum::http::StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn test_require_auth_accepts_authenticated_user() {
            let server = create_require_auth_app();
            let token = token_with("alice", vec!["ROLE_USER".to_string()], false);
            let response = server
                .get("/protected")
                .add_header(AUTHORIZATION, bearer(&token))
                .await;
            response.assert_status_ok();
            assert_eq!(response.text(), "ok");
        }

        // require_role tests: function returns a closure-based middleware.
        fn create_require_role_app(role: &'static str) -> TestServer {
            let state = create_test_state();
            let app = Router::new()
                .route("/admin", get(ok_handler))
                .layer(middleware::from_fn(require_role(role)))
                .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
                .with_state(state);
            TestServer::new(app).unwrap()
        }

        #[tokio::test]
        async fn test_require_role_rejects_user_missing_role() {
            let server = create_require_role_app("ROLE_ADMIN");
            let token = token_with("alice", vec!["ROLE_USER".to_string()], false);
            let response = server
                .get("/admin")
                .add_header(AUTHORIZATION, bearer(&token))
                .expect_failure()
                .await;
            assert_eq!(response.status_code(), axum::http::StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn test_require_role_accepts_user_with_role() {
            let server = create_require_role_app("ROLE_ADMIN");
            let token = token_with("admin", vec!["ROLE_USER".to_string(), "ROLE_ADMIN".to_string()], false);
            let response = server
                .get("/admin")
                .add_header(AUTHORIZATION, bearer(&token))
                .await;
            response.assert_status_ok();
        }

        #[tokio::test]
        async fn test_require_role_rejects_anonymous_user() {
            // No token → anonymous → has_authority returns false → 403.
            let server = create_require_role_app("ROLE_USER");
            let response = server.get("/admin").expect_failure().await;
            assert_eq!(response.status_code(), axum::http::StatusCode::FORBIDDEN);
        }
    }
}
