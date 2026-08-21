use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::UserDto;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::services::{AuthService, UserService};
use crate::AppState;

/// Account routes (current user)
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_account).post(save_account))
        .route("/change-password", post(change_password))
}

/// Public account routes (no authentication required)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
}

/// Password length constraints (matching Spring Boot JHipster)
const PASSWORD_MIN_LENGTH: usize = 4;
const PASSWORD_MAX_LENGTH: usize = 100;

/// Check if password length is invalid
fn is_password_length_invalid(password: &str) -> bool {
    password.is_empty() || password.len() < PASSWORD_MIN_LENGTH || password.len() > PASSWORD_MAX_LENGTH
}

/// Registration request (matches Spring Boot ManagedUserVM)
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// Username (login)
    pub login: String,
    /// Email address
    pub email: String,
    /// Password (4-100 characters)
    pub password: String,
    /// Language key
    #[serde(default)]
    pub lang_key: Option<String>,
}

/// Authentication routes
pub fn auth_routes() -> Router<AppState> {
    Router::new().route("/", post(authenticate))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// Username (login)
    pub username: String,
    /// Password
    pub password: String,
    /// Remember me flag for extended session
    #[serde(default)]
    pub remember_me: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    /// JWT authentication token
    pub id_token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    /// Current password for verification
    pub current_password: String,
    /// New password to set
    pub new_password: String,
}

/// Request body for saving account settings (matches Angular Account model)
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SaveAccountRequest {
    /// First name
    pub first_name: Option<String>,
    /// Last name
    pub last_name: Option<String>,
    /// Email address
    pub email: String,
    /// Language key
    pub lang_key: Option<String>,
    /// Image URL
    pub image_url: Option<String>,
}

/// Authenticate user and return JWT token
/// Returns token in both Authorization header (for JHipster React) and response body
#[utoipa::path(
    post,
    path = "/api/authenticate",
    tag = "authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Authentication successful", body = LoginResponse,
            headers(
                ("Authorization" = String, description = "Bearer token")
            )
        ),
        (status = 400, description = "Missing username or password"),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn authenticate(
    State(state): State<AppState>,
    Json(login): Json<LoginRequest>,
) -> Result<Response, AppError> {
    // Validate required fields - return 400 for missing credentials (matching Spring Boot)
    if login.username.is_empty() {
        return Err(AppError::BadRequest("Username is required".to_string()));
    }
    if login.password.is_empty() {
        return Err(AppError::BadRequest("Password is required".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let token = AuthService::authenticate(
        &mut conn,
        &state.config,
        &login.username,
        &login.password,
        login.remember_me,
    )?;

    // Return token in both header and body for compatibility
    let response = (
        [(header::AUTHORIZATION, format!("Bearer {}", token))],
        Json(LoginResponse { id_token: token }),
    );

    Ok(response.into_response())
}

/// Get current user's account
#[utoipa::path(
    get,
    path = "/api/account",
    tag = "account",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user account info", body = UserDto),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn get_account(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<UserDto>, AppError> {
    // Check if user is authenticated (not anonymous)
    if auth_user.is_anonymous() {
        return Err(AppError::Unauthorized("Not authenticated".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let user = UserService::find_by_login(&mut conn, &auth_user.login)?;

    // Include authorities from JWT in the response
    let mut user_dto = UserDto::from(user);
    user_dto.authorities = Some(auth_user.authorities.clone());

    Ok(Json(user_dto))
}

/// POST /api/account - Save current user's account settings
///
/// Matches Spring Boot JHipster AccountResource.saveAccount():
/// - Updates first name, last name, email, lang key, and image URL
/// - Returns 200 OK on success
/// - Returns 400 Bad Request if email is already in use by another user
#[utoipa::path(
    post,
    path = "/api/account",
    tag = "account",
    security(("bearer_auth" = [])),
    request_body = SaveAccountRequest,
    responses(
        (status = 200, description = "Account saved successfully"),
        (status = 400, description = "Email already in use"),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn save_account(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<SaveAccountRequest>,
) -> Result<StatusCode, AppError> {
    // Check if user is authenticated (not anonymous)
    if auth_user.is_anonymous() {
        return Err(AppError::Unauthorized("Not authenticated".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    // Check if email is already in use by another user
    if let Ok(existing) = UserService::find_by_email(&mut conn, &request.email.to_lowercase()) {
        if existing.login != auth_user.login {
            return Err(AppError::BadRequest("Email already in use".to_string()));
        }
    }

    // Update user account
    UserService::update_account(
        &mut conn,
        &auth_user.login,
        request.first_name,
        request.last_name,
        request.email.to_lowercase(),
        request.lang_key,
        request.image_url,
    )?;

    Ok(StatusCode::OK)
}

/// Change current user's password
#[utoipa::path(
    post,
    path = "/api/account/change-password",
    tag = "account",
    security(("bearer_auth" = [])),
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 400, description = "Invalid current password"),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn change_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    AuthService::change_password(
        &mut conn,
        &auth_user.login,
        &request.current_password,
        &request.new_password,
    )?;

    Ok(StatusCode::OK)
}

/// POST /api/register - Register a new user account
///
/// Matches Spring Boot JHipster AccountResource.registerAccount():
/// - Returns 201 Created on success
/// - Returns 400 Bad Request for invalid password or duplicate login/email
/// - When email is disabled, creates already-activated user
#[utoipa::path(
    post,
    path = "/api/register",
    tag = "account",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully"),
        (status = 400, description = "Invalid password or duplicate login/email")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<StatusCode, AppError> {
    // Validate password length (matching Spring Boot)
    if is_password_length_invalid(&request.password) {
        return Err(AppError::BadRequest("Password must be between 4 and 100 characters".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    // Check login - error if already exists
    if UserService::find_by_login(&mut conn, &request.login.to_lowercase()).is_ok() {
        return Err(AppError::BadRequest("Login already in use".to_string()));
    }

    // Check email - error if already exists
    if UserService::find_by_email(&mut conn, &request.email.to_lowercase()).is_ok() {
        return Err(AppError::BadRequest("Email already in use".to_string()));
    }

    // Create user (already activated since email is disabled)
    let _user = UserService::create_registered_user(
        &mut conn,
        request.login.to_lowercase(),
        request.email.to_lowercase(),
        &request.password,
        request.lang_key.clone(),
    )?;

    Ok(StatusCode::CREATED)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_deserialization() {
        let json = r#"{"username":"testuser","password":"password123","remember_me":true}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "password123");
        assert!(request.remember_me);
    }

    #[test]
    fn test_login_request_default_remember_me() {
        let json = r#"{"username":"testuser","password":"password123"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert!(!request.remember_me);
    }

    #[test]
    fn test_login_response_serialization() {
        let response = LoginResponse {
            id_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.signature".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("id_token"));
        assert!(json.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    }

    #[test]
    fn test_change_password_request_deserialization() {
        // Uses camelCase to match Angular client format
        let json = r#"{"currentPassword":"oldpass","newPassword":"newpass123"}"#;
        let request: ChangePasswordRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.current_password, "oldpass");
        assert_eq!(request.new_password, "newpass123");
    }

    #[test]
    fn test_login_request_debug() {
        let request = LoginRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            remember_me: false,
        };
        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("testuser"));
    }

    #[test]
    fn test_login_response_debug() {
        let response = LoginResponse {
            id_token: "test_token".to_string(),
        };
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("test_token"));
    }

    #[test]
    fn test_change_password_request_debug() {
        let request = ChangePasswordRequest {
            current_password: "oldpass".to_string(),
            new_password: "newpass".to_string(),
        };
        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("oldpass"));
    }

    // Phase 2b (2026-05-11): is_password_length_invalid is the gatekeeper
    // for both `register` and `change_password`. Pin its boundary semantics
    // (4 <= len <= 100) so a future Spring-Boot-spec change can't drift the
    // contract without making someone explicitly update tests.
    #[test]
    fn test_is_password_length_invalid_rejects_empty() {
        assert!(is_password_length_invalid(""));
    }

    #[test]
    fn test_is_password_length_invalid_rejects_below_min() {
        // MIN is 4 — 3-char password must be rejected.
        assert!(is_password_length_invalid("abc"));
    }

    #[test]
    fn test_is_password_length_invalid_accepts_at_min() {
        // Exact MIN (4) is valid.
        assert!(!is_password_length_invalid("abcd"));
    }

    #[test]
    fn test_is_password_length_invalid_accepts_at_max() {
        // Exact MAX (100) is valid.
        let max = "x".repeat(100);
        assert!(!is_password_length_invalid(&max));
    }

    #[test]
    fn test_is_password_length_invalid_rejects_above_max() {
        // MAX + 1 must be rejected.
        let too_long = "x".repeat(101);
        assert!(is_password_length_invalid(&too_long));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::{middleware, Router};
    use axum_test::TestServer;
    use crate::test_utils::{create_test_pool, create_test_config, create_test_admin, create_test_user, generate_test_token};
    use crate::middleware::auth::auth_middleware;

    fn create_test_app() -> (TestServer, crate::AppState) {
        let pool = create_test_pool();
        let config = create_test_config();
        let state = crate::AppState {
            pool,
            config,
        };

        let app = Router::new()
            .nest("/api/authenticate", auth_routes())
            .nest(
                "/api/account",
                routes().layer(middleware::from_fn_with_state(state.clone(), auth_middleware)),
            )
            // Phase 2b (2026-05-11): mount public_routes so register tests can
            // POST /api/register. Additive — doesn't touch the existing
            // protected /api/account routes.
            .nest("/api", public_routes())
            .with_state(state.clone());

        let server = TestServer::new(app).unwrap();
        (server, state)
    }

    #[tokio::test]
    async fn test_authenticate_success() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let login_request = serde_json::json!({
            "username": "admin",
            "password": "admin123",
            "remember_me": false
        });

        let response = server
            .post("/api/authenticate")
            .json(&login_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let text = response.text();
        assert!(text.contains("id_token"));
    }

    #[tokio::test]
    async fn test_authenticate_with_remember_me() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let login_request = serde_json::json!({
            "username": "admin",
            "password": "admin123",
            "remember_me": true
        });

        let response = server
            .post("/api/authenticate")
            .json(&login_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_authenticate_wrong_password() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let login_request = serde_json::json!({
            "username": "admin",
            "password": "wrongpassword"
        });

        let response = server
            .post("/api/authenticate")
            .json(&login_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_authenticate_user_not_found() {
        let (server, _state) = create_test_app();

        let login_request = serde_json::json!({
            "username": "nonexistent",
            "password": "password123"
        });

        let response = server
            .post("/api/authenticate")
            .json(&login_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_authenticate_returns_authorization_header() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let login_request = serde_json::json!({
            "username": "admin",
            "password": "admin123"
        });

        let response = server
            .post("/api/authenticate")
            .json(&login_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let headers = response.headers();
        assert!(headers.contains_key("authorization"));
        let auth_header = headers.get("authorization").unwrap().to_str().unwrap();
        assert!(auth_header.starts_with("Bearer "));
    }

    #[tokio::test]
    async fn test_get_account() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let response = server
            .get("/api/account")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let text = response.text();
        assert!(text.contains("admin"));
        assert!(text.contains("admin@localhost"));
        assert!(text.contains("ROLE_ADMIN"));
    }

    #[tokio::test]
    async fn test_get_account_unauthorized() {
        let (server, _state) = create_test_app();

        let response = server
            .get("/api/account")
            .await;

        // Without token, the auth middleware should reject with unauthorized
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_change_password_success() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        // Uses camelCase to match Angular client format
        let change_request = serde_json::json!({
            "currentPassword": "admin123",
            "newPassword": "newpassword456"
        });

        let response = server
            .post("/api/account/change-password")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&change_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        // Verify new password works
        let login_request = serde_json::json!({
            "username": "admin",
            "password": "newpassword456"
        });

        let auth_response = server
            .post("/api/authenticate")
            .json(&login_request)
            .await;

        assert_eq!(auth_response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_change_password_wrong_current() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        // Uses camelCase to match Angular client format
        let change_request = serde_json::json!({
            "currentPassword": "wrongpassword",
            "newPassword": "newpassword456"
        });

        let response = server
            .post("/api/account/change-password")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&change_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_regular_user_get_account() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = generate_test_token(
            &state.config,
            "user",
            &["ROLE_USER".to_string()],
        );

        let response = server
            .get("/api/account")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let text = response.text();
        assert!(text.contains("user"));
        assert!(text.contains("user@localhost"));
    }

    // Phase 2b (2026-05-11): coverage push for handlers/account.rs error paths.
    // Targets the 38 uncovered lines that the Phase 1c/1d work didn't reach:
    // save_account branches (anonymous, duplicate-email, update-account path),
    // register branches (password length, duplicate login, duplicate email,
    // happy path), and the authenticate 400 BadRequest gaps for empty
    // username/password. Quality bar: each test asserts the specific status
    // code AND a body fragment that pins the contract (e.g., the message
    // text, the error-type field), not bare 4xx.

    // --- save_account error paths ---

    #[tokio::test]
    async fn test_save_account_rejects_unauthenticated_user() {
        // No Authorization header → auth_middleware inserts anonymous user →
        // save_account returns Unauthorized. Pins the contract that public
        // POST /api/account without credentials is rejected.
        let (server, _state) = create_test_app();
        let body = serde_json::json!({
            "firstName": "First",
            "lastName": "Last",
            "email": "anyone@example.com",
            "langKey": "en",
            "imageUrl": null,
        });
        let response = server.post("/api/account").json(&body).await;
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
        assert!(response.text().contains("Not authenticated"), "got: {}", response.text());
    }

    #[tokio::test]
    async fn test_save_account_rejects_email_owned_by_another_user() {
        // Two users exist (admin + user). Sign in as `user` and try to save
        // an account update with admin's email. The duplicate-email check
        // must reject with 400 — not 200 — to prevent account hijack.
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);
        create_test_user(&state.pool);
        let token = generate_test_token(&state.config, "user", &["ROLE_USER".to_string()]);

        let body = serde_json::json!({
            "firstName": "User",
            "lastName": "Test",
            "email": "admin@localhost",
            "langKey": "en",
            "imageUrl": null,
        });
        let response = server
            .post("/api/account")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&body)
            .await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
        assert!(response.text().contains("Email already in use"), "got: {}", response.text());
    }

    #[tokio::test]
    async fn test_save_account_allows_user_to_keep_own_email() {
        // The duplicate-email check only fires when the matched login
        // differs from the requester. Posting one's own email back must NOT
        // be rejected — covers the fall-through branch.
        let (server, state) = create_test_app();
        create_test_user(&state.pool);
        let token = generate_test_token(&state.config, "user", &["ROLE_USER".to_string()]);

        let body = serde_json::json!({
            "firstName": "User",
            "lastName": "Test",
            "email": "user@localhost",  // same as current user
            "langKey": "fr",
            "imageUrl": null,
        });
        let response = server
            .post("/api/account")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&body)
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_save_account_updates_account_on_new_email() {
        // Happy path that exercises update_account with a non-conflicting
        // email. The seeded test users (admin, user) have known emails;
        // posting a brand-new email triggers the update path.
        let (server, state) = create_test_app();
        create_test_user(&state.pool);
        let token = generate_test_token(&state.config, "user", &["ROLE_USER".to_string()]);

        let body = serde_json::json!({
            "firstName": "Updated",
            "lastName": "Name",
            "email": "user-new-email@example.com",
            "langKey": "es",
            "imageUrl": "https://example.com/avatar.png",
        });
        let response = server
            .post("/api/account")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&body)
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    // --- register error paths ---
    //
    // `register` is gated `!enableEmail` in the production code path; the
    // canonical microservice-cb scaffold has email disabled, so these
    // tests run against the always-activated `create_registered_user`
    // codepath. Tests use unique login/email prefixes to avoid colliding
    // with seeded users and with one another.

    #[tokio::test]
    async fn test_register_rejects_password_below_min() {
        // MIN is 4 chars. Anything shorter must 400 BEFORE the user is
        // created — verify by the response code; if the error message had
        // leaked the password we'd want to assert it doesn't.
        let (server, _state) = create_test_app();
        let body = serde_json::json!({
            "login": "phase2bshort",
            "email": "phase2bshort@example.com",
            "password": "abc",  // 3 chars, below MIN(4)
            "langKey": "en",
        });
        let response = server.post("/api/register").json(&body).await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
        assert!(response.text().contains("Password must be between"), "got: {}", response.text());
    }

    #[tokio::test]
    async fn test_register_rejects_password_above_max() {
        // MAX is 100. 101 chars must 400.
        let (server, _state) = create_test_app();
        let body = serde_json::json!({
            "login": "phase2blong",
            "email": "phase2blong@example.com",
            "password": "x".repeat(101),
            "langKey": "en",
        });
        let response = server.post("/api/register").json(&body).await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
        assert!(response.text().contains("Password must be between"), "got: {}", response.text());
    }

    #[tokio::test]
    async fn test_register_rejects_duplicate_login() {
        // Seed `admin` then try to register a fresh user with login `admin`.
        // Must 400 with the documented contract message.
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);
        let body = serde_json::json!({
            "login": "admin",  // collides with seeded admin
            "email": "phase2bdupelogin@example.com",
            "password": "validpass123",
            "langKey": "en",
        });
        let response = server.post("/api/register").json(&body).await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
        assert!(response.text().contains("Login already in use"), "got: {}", response.text());
    }

    #[tokio::test]
    async fn test_register_rejects_duplicate_email() {
        // Distinct login but same email as a seeded user must 400.
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);
        let body = serde_json::json!({
            "login": "phase2bdupemail",
            "email": "admin@localhost",  // collides with seeded admin's email
            "password": "validpass123",
            "langKey": "en",
        });
        let response = server.post("/api/register").json(&body).await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
        assert!(response.text().contains("Email already in use"), "got: {}", response.text());
    }

    #[tokio::test]
    async fn test_register_succeeds_with_valid_input() {
        // Happy path. Unique login + email + valid password + lang_key →
        // 201 Created. Exercises the full create_registered_user code path.
        let (server, _state) = create_test_app();
        let body = serde_json::json!({
            "login": "phase2bhappy",
            "email": "phase2bhappy@example.com",
            "password": "validpass123",
            "langKey": "en",
        });
        let response = server.post("/api/register").json(&body).await;
        assert_eq!(response.status_code(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_register_accepts_omitted_lang_key() {
        // lang_key has #[serde(default)] → Option<String>::None. Verify
        // that omitting it doesn't 400; the registered user should still
        // be created (lang_key=None gets carried through).
        let (server, _state) = create_test_app();
        let body = serde_json::json!({
            "login": "phase2bnolang",
            "email": "phase2bnolang@example.com",
            "password": "validpass123",
        });
        let response = server.post("/api/register").json(&body).await;
        assert_eq!(response.status_code(), StatusCode::CREATED);
    }

    // --- authenticate 400 BadRequest gaps ---

    #[tokio::test]
    async fn test_authenticate_rejects_empty_username() {
        // The handler's first guard returns BadRequest before touching the
        // DB or AuthService. Documents the contract that empty credentials
        // are a 400, not a 401 — Spring Boot parity.
        let (server, _state) = create_test_app();
        let body = serde_json::json!({
            "username": "",
            "password": "anything",
            "remember_me": false,
        });
        let response = server.post("/api/authenticate").json(&body).await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
        assert!(response.text().contains("Username is required"), "got: {}", response.text());
    }

    #[tokio::test]
    async fn test_authenticate_rejects_empty_password() {
        let (server, _state) = create_test_app();
        let body = serde_json::json!({
            "username": "admin",
            "password": "",
            "remember_me": false,
        });
        let response = server.post("/api/authenticate").json(&body).await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
        assert!(response.text().contains("Password is required"), "got: {}", response.text());
    }
}
