use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};

use crate::dto::{OfferTailoredResumeDto, CreateOfferTailoredResumeDto, UpdateOfferTailoredResumeDto, PageRequest, QsQuery};
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::RoleType;
use crate::services::OfferTailoredResumeService;
use crate::AppState;

/// OfferTailoredResume routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/:id", get(get_one).put(update).delete(remove))
}

/// Get all offerTailoredResumes with pagination
#[utoipa::path(
    get,
    path = "/api/offer-tailored-resumes",
    tag = "offer-tailored-resumes",
    security(("bearer_auth" = [])),
    params(
        ("page" = Option<i64>, Query, description = "Page number (0-indexed)"),
        ("size" = Option<i64>, Query, description = "Page size"),
        ("sort" = Option<String>, Query, description = "Sort field and direction (e.g., 'id,asc')")
    ),
    responses(
        (status = 200, description = "List of offerTailoredResumes with pagination", body = Vec<OfferTailoredResumeDto>,
            headers(
                ("X-Total-Count" = i64, description = "Total number of items")
            )
        ),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_all(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    QsQuery(page_request): QsQuery<PageRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !auth_user.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let (items, total) = OfferTailoredResumeService::find_all(&mut conn, &page_request)?;

    let dtos: Vec<OfferTailoredResumeDto> = items
        .into_iter()
        .map(|item| {
            let jobOffer = item.jobOffer_id
                .and_then(|id| OfferTailoredResumeService::find_jobOffer_by_id(&mut conn, id).ok().flatten());
            OfferTailoredResumeDto::from_with_relations(item, jobOffer)
        })
        .collect();

    let mut headers = HeaderMap::new();
    headers.insert("X-Total-Count", HeaderValue::from_str(&total.to_string()).unwrap());

    Ok((headers, Json(dtos)))
}

/// Get offerTailoredResume by ID
#[utoipa::path(
    get,
    path = "/api/offer-tailored-resumes/{id}",
    tag = "offer-tailored-resumes",
    security(("bearer_auth" = [])),
    params(
        ("id" = i32, Path, description = "OfferTailoredResume ID")
    ),
    responses(
        (status = 200, description = "OfferTailoredResume found", body = OfferTailoredResumeDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "OfferTailoredResume not found")
    )
)]
pub async fn get_one(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> Result<Json<OfferTailoredResumeDto>, AppError> {
    if !auth_user.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let item = OfferTailoredResumeService::find_by_id(&mut conn, id)?;

    // Fetch related entities
    let jobOffer = item.jobOffer_id
        .and_then(|id| OfferTailoredResumeService::find_jobOffer_by_id(&mut conn, id).ok().flatten());

    Ok(Json(OfferTailoredResumeDto::from_with_relations(item, jobOffer)))
}

/// Create a new offerTailoredResume
#[utoipa::path(
    post,
    path = "/api/offer-tailored-resumes",
    tag = "offer-tailored-resumes",
    security(("bearer_auth" = [])),
    request_body = CreateOfferTailoredResumeDto,
    responses(
        (status = 201, description = "OfferTailoredResume created successfully", body = OfferTailoredResumeDto),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<CreateOfferTailoredResumeDto>,
) -> Result<(StatusCode, Json<OfferTailoredResumeDto>), AppError> {
    if !auth_user.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let item = OfferTailoredResumeService::create(&mut conn, dto, &auth_user.login)?;


    // Fetch related entities for response
    let jobOffer = item.jobOffer_id
        .and_then(|id| OfferTailoredResumeService::find_jobOffer_by_id(&mut conn, id).ok().flatten());

    Ok((StatusCode::CREATED, Json(OfferTailoredResumeDto::from_with_relations(item, jobOffer))))
}

/// Update an existing offerTailoredResume
#[utoipa::path(
    put,
    path = "/api/offer-tailored-resumes/{id}",
    tag = "offer-tailored-resumes",
    security(("bearer_auth" = [])),
    params(
        ("id" = i32, Path, description = "OfferTailoredResume ID")
    ),
    request_body = UpdateOfferTailoredResumeDto,
    responses(
        (status = 200, description = "OfferTailoredResume updated successfully", body = OfferTailoredResumeDto),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "OfferTailoredResume not found")
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    Json(dto): Json<UpdateOfferTailoredResumeDto>,
) -> Result<Json<OfferTailoredResumeDto>, AppError> {
    if !auth_user.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let item = OfferTailoredResumeService::update(&mut conn, id, dto, &auth_user.login)?;


    // Fetch related entities for response
    let jobOffer = item.jobOffer_id
        .and_then(|id| OfferTailoredResumeService::find_jobOffer_by_id(&mut conn, id).ok().flatten());

    Ok(Json(OfferTailoredResumeDto::from_with_relations(item, jobOffer)))
}

/// Delete a offerTailoredResume
#[utoipa::path(
    delete,
    path = "/api/offer-tailored-resumes/{id}",
    tag = "offer-tailored-resumes",
    security(("bearer_auth" = [])),
    params(
        ("id" = i32, Path, description = "OfferTailoredResume ID")
    ),
    responses(
        (status = 204, description = "OfferTailoredResume deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "OfferTailoredResume not found")
    )
)]
pub async fn remove(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    if !auth_user.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    OfferTailoredResumeService::delete(&mut conn, id)?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::middleware;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use crate::test_utils::{create_test_pool, create_test_config, create_test_user, generate_test_token};
    use crate::middleware::auth::auth_middleware;

    fn create_test_app() -> (TestServer, crate::AppState) {
        let pool = create_test_pool();
        let config = create_test_config();
        let state = crate::AppState {
            pool,
            config,
        };

        let app = Router::new()
            .nest("/api/offer-tailored-resumes", routes())
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state.clone());

        let server = TestServer::new(app).unwrap();
        (server, state)
    }

    fn get_auth_token(state: &crate::AppState) -> String {
        generate_test_token(
            &state.config,
            "user",
            &["ROLE_USER".to_string()],
        )
    }

    #[tokio::test]
    async fn test_get_all_offerTailoredResumes() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = get_auth_token(&state);

        let response = server
            .get("/api/offer-tailored-resumes")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_all_offerTailoredResumes_unauthorized() {
        let (server, _state) = create_test_app();

        let response = server
            .get("/api/offer-tailored-resumes")
            .await;

        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_create_offerTailoredResume() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = get_auth_token(&state);

        let new_entity = serde_json::json!({
            "userId": "test_value",
            "data": "test_value",
        });

        let response = server
            .post("/api/offer-tailored-resumes")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&new_entity)
            .await;

        assert_eq!(response.status_code(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_get_offerTailoredResume_not_found() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = get_auth_token(&state);

        let response = server
            .get("/api/offer-tailored-resumes/99999")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_and_get_offerTailoredResume() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = get_auth_token(&state);

        // Create a new entity
        let new_entity = serde_json::json!({
            "userId": "test_create_get",
            "data": "test_create_get"
        });

        let create_response = server
            .post("/api/offer-tailored-resumes")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&new_entity)
            .await;

        assert_eq!(create_response.status_code(), StatusCode::CREATED);
        let created: serde_json::Value = create_response.json();
        let id = created["id"].as_i64().unwrap();

        // Get the created entity
        let get_response = server
            .get(&format!("/api/offer-tailored-resumes/{}", id))
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(get_response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_offerTailoredResume() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = get_auth_token(&state);

        // First create an entity
        let new_entity = serde_json::json!({
            "userId": "original_value",
            "data": "original_value"
        });

        let create_response = server
            .post("/api/offer-tailored-resumes")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&new_entity)
            .await;

        assert_eq!(create_response.status_code(), StatusCode::CREATED);
        let created: serde_json::Value = create_response.json();
        let id = created["id"].as_i64().unwrap();

        // Update the entity
        let update_data = serde_json::json!({
            "userId": "updated_value"
        });

        let update_response = server
            .put(&format!("/api/offer-tailored-resumes/{}", id))
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&update_data)
            .await;

        assert_eq!(update_response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_offerTailoredResume() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = get_auth_token(&state);

        // First create an entity
        let new_entity = serde_json::json!({
            "userId": "to_be_deleted",
            "data": "to_be_deleted"
        });

        let create_response = server
            .post("/api/offer-tailored-resumes")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .json(&new_entity)
            .await;

        assert_eq!(create_response.status_code(), StatusCode::CREATED);
        let created: serde_json::Value = create_response.json();
        let id = created["id"].as_i64().unwrap();

        // Delete the entity
        let delete_response = server
            .delete(&format!("/api/offer-tailored-resumes/{}", id))
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);

        // Verify it's deleted
        let get_response = server
            .get(&format!("/api/offer-tailored-resumes/{}", id))
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_pagination_headers() {
        let (server, state) = create_test_app();
        create_test_user(&state.pool);

        let token = get_auth_token(&state);

        let response = server
            .get("/api/offer-tailored-resumes")
            .add_query_param("page", "0")
            .add_query_param("size", "10")
            .add_header("Authorization".parse().unwrap(), format!("Bearer {}", token).parse().unwrap())
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let headers = response.headers();
        assert!(headers.contains_key("x-total-count"));
    }
}
