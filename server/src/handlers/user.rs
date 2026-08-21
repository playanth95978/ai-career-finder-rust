use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::dto::{CreateUserDto, PageRequest, QsQuery, UpdateUserDto, UserDto};
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::RoleType;
use crate::services::UserService;
use crate::AppState;

/// Public user routes (limited info)
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_public_users))
}

/// Admin user management routes
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_users).post(create_user).put(update_user_from_body))
        .route("/:login", get(get_user).delete(delete_user))
}

/// Authority routes (full CRUD; admin only). Bug #14 fix from 1-a.5.0:
/// prior version was GET-only with a hard-coded ROLE_ADMIN/ROLE_USER vec,
/// so the cypress entity/authority.cy.ts test (POST /api/authorities) got
/// 405 Method Not Allowed.
pub fn authority_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_authorities).post(create_authority))
        .route("/:name", get(get_authority).delete(delete_authority))
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct AuthorityDto {
    /// Authority name (e.g., ROLE_ADMIN, ROLE_USER)
    #[validate(length(min = 1, max = 50))]
    pub name: String,
}

/// Get all authorities (admin only)
#[utoipa::path(
    get,
    path = "/api/authorities",
    tag = "user-management",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of all authorities", body = Vec<AuthorityDto>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    )
)]
pub async fn get_authorities(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<AuthorityDto>>, AppError> {
    if !auth_user.has_authority(RoleType::ADMIN) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let names = UserService::find_all_authorities(&mut conn)?;
    Ok(Json(names.into_iter().map(|name| AuthorityDto { name }).collect()))
}

/// Get a single authority by name (admin only). Bug #14 follow-on:
/// the Angular entity-detail view fetches /:name and a 404 made the
/// front-end's error handler null-deref on `.message`.
#[utoipa::path(
    get,
    path = "/api/authorities/{name}",
    tag = "user-management",
    security(("bearer_auth" = [])),
    params(
        ("name" = String, Path, description = "Authority name")
    ),
    responses(
        (status = 200, description = "Authority found", body = AuthorityDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Authority not found")
    )
)]
pub async fn get_authority(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<Json<AuthorityDto>, AppError> {
    if !auth_user.has_authority(RoleType::ADMIN) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let names = UserService::find_all_authorities(&mut conn)?;
    names.into_iter()
        .find(|n| n == &name)
        .map(|name| Json(AuthorityDto { name }))
        .ok_or_else(|| AppError::NotFound(format!("Authority '{}' not found", name)))
}

/// Create a new authority (admin only)
#[utoipa::path(
    post,
    path = "/api/authorities",
    tag = "user-management",
    security(("bearer_auth" = [])),
    request_body = AuthorityDto,
    responses(
        (status = 201, description = "Authority created", body = AuthorityDto),
        (status = 400, description = "Invalid name or already exists"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    )
)]
pub async fn create_authority(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<AuthorityDto>,
) -> Result<(StatusCode, Json<AuthorityDto>), AppError> {
    if !auth_user.has_authority(RoleType::ADMIN) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    dto.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    UserService::create_authority(&mut conn, &dto.name)?;
    Ok((StatusCode::CREATED, Json(dto)))
}

/// Delete an authority (admin only)
#[utoipa::path(
    delete,
    path = "/api/authorities/{name}",
    tag = "user-management",
    security(("bearer_auth" = [])),
    params(
        ("name" = String, Path, description = "Authority name")
    ),
    responses(
        (status = 204, description = "Authority deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Authority not found")
    )
)]
pub async fn delete_authority(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    if !auth_user.has_authority(RoleType::ADMIN) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    UserService::delete_authority(&mut conn, &name)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get public user list (limited info, for non-admin users)
async fn get_public_users(
    State(state): State<AppState>,
    QsQuery(page_request): QsQuery<PageRequest>,
) -> Result<Response, AppError> {
    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let (users, total) = UserService::find_all(&mut conn, &page_request)?;

    let user_dtos: Vec<UserDto> = users.into_iter().map(UserDto::from).collect();

    let response = (
        [(header::HeaderName::from_static("x-total-count"), total.to_string())],
        Json(user_dtos),
    );

    Ok(response.into_response())
}

/// Get all users with pagination (admin only)
#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = "user-management",
    security(("bearer_auth" = [])),
    params(
        ("page" = Option<i64>, Query, description = "Page number (0-indexed)"),
        ("size" = Option<i64>, Query, description = "Page size"),
        ("sort" = Option<String>, Query, description = "Sort field and direction (e.g., 'id,asc')")
    ),
    responses(
        (status = 200, description = "List of users with pagination", body = Vec<UserDto>,
            headers(
                ("x-total-count" = i64, description = "Total number of users")
            )
        ),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    )
)]
pub async fn get_all_users(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    QsQuery(page_request): QsQuery<PageRequest>,
) -> Result<Response, AppError> {
    if !auth_user.has_authority(RoleType::ADMIN) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let (users, total) = UserService::find_all(&mut conn, &page_request)?;

    let user_dtos: Vec<UserDto> = users.into_iter().map(|user| {
        let mut dto = UserDto::from(user.clone());
        // Load authorities for admin view
        if let Ok(authorities) = UserService::get_authorities(&mut conn, user.id) {
            dto.authorities = Some(authorities);
        }
        dto
    }).collect();

    let response = (
        [(header::HeaderName::from_static("x-total-count"), total.to_string())],
        Json(user_dtos),
    );

    Ok(response.into_response())
}

/// Get user by login (admin only)
#[utoipa::path(
    get,
    path = "/api/admin/users/{login}",
    tag = "user-management",
    security(("bearer_auth" = [])),
    params(
        ("login" = String, Path, description = "User login")
    ),
    responses(
        (status = 200, description = "User found", body = UserDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(login): Path<String>,
) -> Result<Json<UserDto>, AppError> {
    if !auth_user.has_authority(RoleType::ADMIN) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let user = UserService::find_by_login(&mut conn, &login)?;
    let mut user_dto = UserDto::from(user.clone());

    // Load authorities
    if let Ok(authorities) = UserService::get_authorities(&mut conn, user.id) {
        user_dto.authorities = Some(authorities);
    }

    Ok(Json(user_dto))
}

/// Create a new user (admin only)
#[utoipa::path(
    post,
    path = "/api/admin/users",
    tag = "user-management",
    security(("bearer_auth" = [])),
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User created successfully", body = UserDto),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    )
)]
pub async fn create_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(create_dto): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<UserDto>), AppError> {
    if !auth_user.has_authority(RoleType::ADMIN) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let user = UserService::create(&mut conn, create_dto, &auth_user.login)?;

    Ok((StatusCode::CREATED, Json(UserDto::from(user))))
}

/// Update an existing user (admin only) - uses login from request body
/// This endpoint is used by JHipster Angular UI which sends the full user object
#[utoipa::path(
    put,
    path = "/api/admin/users",
    tag = "user-management",
    security(("bearer_auth" = [])),
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "User updated successfully", body = UserDto),
        (status = 400, description = "Invalid input or missing login"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    )
)]
pub async fn update_user_from_body(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(update_dto): Json<UpdateUserDto>,
) -> Result<Json<UserDto>, AppError> {
    if !auth_user.has_authority(RoleType::ADMIN) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // Extract login from the body
    let login = update_dto.login.clone()
        .ok_or_else(|| AppError::BadRequest("Login is required".to_string()))?;

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let user = UserService::find_by_login(&mut conn, &login)?;
    let updated_user = UserService::update(&mut conn, user.id, update_dto, &auth_user.login)?;

    Ok(Json(UserDto::from(updated_user)))
}

/// Delete a user (admin only)
#[utoipa::path(
    delete,
    path = "/api/admin/users/{login}",
    tag = "user-management",
    security(("bearer_auth" = [])),
    params(
        ("login" = String, Path, description = "User login")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    )
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(login): Path<String>,
) -> Result<StatusCode, AppError> {
    if !auth_user.has_authority(RoleType::ADMIN) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let user = UserService::find_by_login(&mut conn, &login)?;
    UserService::delete(&mut conn, user.id)?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authority_dto_serialization() {
        let dto = AuthorityDto {
            name: "ROLE_ADMIN".to_string(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("ROLE_ADMIN"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::middleware;
    use axum::http::StatusCode;
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
            .nest("/api/users", routes())
            .nest("/api/admin/users", admin_routes())
            .nest("/api/authorities", authority_routes())
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state.clone());

        let server = TestServer::new(app).unwrap();
        (server, state)
    }

    #[tokio::test]
    async fn test_get_public_users() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let response = server
            .get("/api/users")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_all_users_as_admin() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);
        create_test_user(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let response = server
            .get("/api/admin/users")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let text = response.text();
        assert!(text.contains("admin"));
    }

    #[tokio::test]
    async fn test_get_all_users_forbidden_for_regular_user() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = generate_test_token(
            &state.config,
            "user",
            &["ROLE_USER".to_string()],
        );

        let response = server
            .get("/api/admin/users")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_user_by_login() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let response = server
            .get("/api/admin/users/admin")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let text = response.text();
        assert!(text.contains("admin"));
        assert!(text.contains("admin@localhost"));
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let response = server
            .get("/api/admin/users/nonexistent")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_user() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let new_user = serde_json::json!({
            "login": "newuser",
            "password": "password123",
            "email": "newuser@example.com",
            "firstName": "New",
            "lastName": "User",
            "activated": true,
            "langKey": "en"
        });

        let response = server
            .post("/api/admin/users")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&new_user)
            .await;

        assert_eq!(response.status_code(), StatusCode::CREATED);
        let text = response.text();
        assert!(text.contains("newuser"));
    }

    #[tokio::test]
    async fn test_create_user_forbidden_for_regular_user() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = generate_test_token(
            &state.config,
            "user",
            &["ROLE_USER".to_string()],
        );

        let new_user = serde_json::json!({
            "login": "hackeruser",
            "password": "password123",
            "email": "hacker@example.com"
        });

        let response = server
            .post("/api/admin/users")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&new_user)
            .await;

        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_update_user() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);
        create_test_user(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        // JHipster Angular UI sends PUT /api/admin/users with login in body
        let update_data = serde_json::json!({
            "login": "user",
            "firstName": "Updated",
            "lastName": "Name"
        });

        let response = server
            .put("/api/admin/users")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&update_data)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let text = response.text();
        assert!(text.contains("Updated"));
    }

    #[tokio::test]
    async fn test_delete_user() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);
        create_test_user(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let response = server
            .delete("/api/admin/users/user")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

        // Verify user is deleted
        let get_response = server
            .get("/api/admin/users/user")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_authorities_as_admin() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let response = server
            .get("/api/authorities")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let text = response.text();
        assert!(text.contains("ROLE_ADMIN"));
        assert!(text.contains("ROLE_USER"));
    }

    #[tokio::test]
    async fn test_get_authorities_forbidden_for_regular_user() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = generate_test_token(
            &state.config,
            "user",
            &["ROLE_USER".to_string()],
        );

        let response = server
            .get("/api/authorities")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    }

    // ---- Authority CRUD (bug #14 fix from 1-a.5.0) -----------------

    #[tokio::test]
    async fn test_create_authority_as_admin() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let response = server
            .post("/api/authorities")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&serde_json::json!({ "name": "ROLE_REPORTER" }))
            .await;

        assert_eq!(response.status_code(), StatusCode::CREATED);
        let body: serde_json::Value = response.json();
        assert_eq!(body["name"], "ROLE_REPORTER");

        // Confirm it now appears in GET.
        let list = server
            .get("/api/authorities")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;
        assert!(list.text().contains("ROLE_REPORTER"));
    }

    #[tokio::test]
    async fn test_create_authority_duplicate_returns_400() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        // ROLE_ADMIN is seeded by migrations.
        let response = server
            .post("/api/authorities")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&serde_json::json!({ "name": "ROLE_ADMIN" }))
            .await;

        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_authority_forbidden_for_regular_user() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = generate_test_token(
            &state.config,
            "user",
            &["ROLE_USER".to_string()],
        );

        let response = server
            .post("/api/authorities")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&serde_json::json!({ "name": "ROLE_SHOULD_NOT_EXIST" }))
            .await;

        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_delete_authority_as_admin() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        // Create then delete a non-seeded authority (avoid FK conflicts with
        // the admin's existing ROLE_ADMIN/ROLE_USER assignments).
        server
            .post("/api/authorities")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&serde_json::json!({ "name": "ROLE_TO_DELETE" }))
            .await;

        let response = server
            .delete("/api/authorities/ROLE_TO_DELETE")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_get_authority_by_name_as_admin() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        // ROLE_ADMIN is seeded by migrations.
        let response = server
            .get("/api/authorities/ROLE_ADMIN")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let body: serde_json::Value = response.json();
        assert_eq!(body["name"], "ROLE_ADMIN");
    }

    #[tokio::test]
    async fn test_get_authority_by_name_not_found() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let response = server
            .get("/api/authorities/ROLE_NEVER_EXISTED")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_authority_not_found() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        let response = server
            .delete("/api/authorities/ROLE_NEVER_EXISTED")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_unauthorized_access_without_token() {
        let (server, _state) = create_test_app();

        let response = server
            .get("/api/admin/users")
            .await;

        // Without token, anonymous user doesn't have ROLE_ADMIN so gets forbidden
        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_pagination_headers() {
        let (server, state) = create_test_app();
        create_test_admin(&state.pool);
        create_test_user(&state.pool);

        let token = generate_test_token(
            &state.config,
            "admin",
            &["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()],
        );

        // Use separate query parameters instead of URL-encoded string
        let response = server
            .get("/api/admin/users")
            .add_query_param("page", "0")
            .add_query_param("size", "10")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        // Check x-total-count header exists
        let headers = response.headers();
        assert!(headers.contains_key("x-total-count"));
    }
}
