use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Error response body
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error type (e.g., "Bad Request", "Not Found")
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<String>>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "Bad Request", msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "Unauthorized", msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "Forbidden", msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "Not Found", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "Conflict", msg.clone()),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, "Validation Error", msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                    "An internal error occurred".to_string(),
                )
            }
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
            details: None,
        });

        (status, body).into_response()
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => AppError::NotFound("Resource not found".to_string()),
            diesel::result::Error::DatabaseError(kind, info) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => {
                    AppError::Conflict(info.message().to_string())
                }
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => {
                    AppError::BadRequest(info.message().to_string())
                }
                _ => AppError::Internal(info.message().to_string()),
            },
            _ => AppError::Internal(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_bad_request_message() {
        let error = AppError::BadRequest("Invalid input".to_string());
        assert_eq!(error.to_string(), "Bad request: Invalid input");
    }

    #[test]
    fn test_app_error_unauthorized_message() {
        let error = AppError::Unauthorized("Invalid credentials".to_string());
        assert_eq!(error.to_string(), "Unauthorized: Invalid credentials");
    }

    #[test]
    fn test_app_error_forbidden_message() {
        let error = AppError::Forbidden("Access denied".to_string());
        assert_eq!(error.to_string(), "Forbidden: Access denied");
    }

    #[test]
    fn test_app_error_not_found_message() {
        let error = AppError::NotFound("Resource not found".to_string());
        assert_eq!(error.to_string(), "Not found: Resource not found");
    }

    #[test]
    fn test_app_error_conflict_message() {
        let error = AppError::Conflict("Resource already exists".to_string());
        assert_eq!(error.to_string(), "Conflict: Resource already exists");
    }

    #[test]
    fn test_app_error_validation_message() {
        let error = AppError::Validation("Email is invalid".to_string());
        assert_eq!(error.to_string(), "Validation error: Email is invalid");
    }

    #[test]
    fn test_app_error_internal_message() {
        let error = AppError::Internal("Database connection failed".to_string());
        assert_eq!(error.to_string(), "Internal error: Database connection failed");
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse {
            error: "Bad Request".to_string(),
            message: "Invalid input".to_string(),
            details: Some(vec!["Field 'email' is required".to_string()]),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"error\":\"Bad Request\""));
        assert!(json.contains("\"message\":\"Invalid input\""));
        assert!(json.contains("\"details\""));
    }

    #[test]
    fn test_error_response_without_details() {
        let response = ErrorResponse {
            error: "Not Found".to_string(),
            message: "User not found".to_string(),
            details: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("details"));
    }

    #[test]
    fn test_diesel_not_found_conversion() {
        let diesel_error = diesel::result::Error::NotFound;
        let app_error: AppError = diesel_error.into();
        match app_error {
            AppError::NotFound(msg) => assert_eq!(msg, "Resource not found"),
            _ => panic!("Expected NotFound error"),
        }
    }

    // Track 1 Phase 1d (2026-05-11): IntoResponse coverage. Each AppError
    // variant must produce its documented status + body shape, since
    // downstream clients (Angular ErrorInterceptor, JHipster bug-handlers)
    // key on these specific values. Test asserts status code AND the
    // error type string in the JSON body AND the message text — not just
    // that into_response() returns a Response (that's a tautology).
    mod into_response_tests {
        use super::super::*;
        use axum::body::to_bytes;
        use axum::http::StatusCode;
        use axum::response::IntoResponse;

        async fn body_string(response: axum::response::Response) -> (StatusCode, String) {
            let status = response.status();
            let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
            (status, String::from_utf8(bytes.to_vec()).unwrap())
        }

        #[tokio::test]
        async fn test_bad_request_into_response_returns_400_with_body() {
            let err = AppError::BadRequest("invalid field".to_string());
            let (status, body) = body_string(err.into_response()).await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert!(body.contains("\"error\":\"Bad Request\""), "got: {}", body);
            assert!(body.contains("\"message\":\"invalid field\""), "got: {}", body);
        }

        #[tokio::test]
        async fn test_unauthorized_into_response_returns_401() {
            let err = AppError::Unauthorized("bad token".to_string());
            let (status, body) = body_string(err.into_response()).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert!(body.contains("\"error\":\"Unauthorized\""));
            assert!(body.contains("\"message\":\"bad token\""));
        }

        #[tokio::test]
        async fn test_forbidden_into_response_returns_403() {
            let err = AppError::Forbidden("role required".to_string());
            let (status, body) = body_string(err.into_response()).await;
            assert_eq!(status, StatusCode::FORBIDDEN);
            assert!(body.contains("\"error\":\"Forbidden\""));
            assert!(body.contains("\"message\":\"role required\""));
        }

        #[tokio::test]
        async fn test_not_found_into_response_returns_404() {
            let err = AppError::NotFound("user 42".to_string());
            let (status, body) = body_string(err.into_response()).await;
            assert_eq!(status, StatusCode::NOT_FOUND);
            assert!(body.contains("\"error\":\"Not Found\""));
            assert!(body.contains("\"message\":\"user 42\""));
        }

        #[tokio::test]
        async fn test_conflict_into_response_returns_409() {
            let err = AppError::Conflict("duplicate login".to_string());
            let (status, body) = body_string(err.into_response()).await;
            assert_eq!(status, StatusCode::CONFLICT);
            assert!(body.contains("\"error\":\"Conflict\""));
            assert!(body.contains("\"message\":\"duplicate login\""));
        }

        #[tokio::test]
        async fn test_validation_into_response_returns_422() {
            let err = AppError::Validation("email format".to_string());
            let (status, body) = body_string(err.into_response()).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert!(body.contains("\"error\":\"Validation Error\""));
            assert!(body.contains("\"message\":\"email format\""));
        }

        #[tokio::test]
        async fn test_internal_into_response_returns_500_with_generic_message() {
            // Documented contract: Internal errors NEVER leak the original
            // message to the client — they're logged server-side and the
            // client sees a generic "An internal error occurred" message.
            let err = AppError::Internal("DB password leaked".to_string());
            let (status, body) = body_string(err.into_response()).await;
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
            assert!(body.contains("\"error\":\"Internal Server Error\""));
            assert!(body.contains("\"message\":\"An internal error occurred\""));
            // Crucial: the sensitive original message must NOT appear in the
            // response body — only in the server-side tracing::error log.
            assert!(!body.contains("DB password leaked"), "leaked internal detail: {}", body);
        }
    }

    // Diesel error conversion: covers the kinds of database errors a
    // generated service emits in normal operation, plus the catch-all
    // for kinds the match arm doesn't explicitly handle.
    mod diesel_conversion_tests {
        use super::super::*;
        use diesel::result::{DatabaseErrorKind, Error as DieselError};

        // Minimal stub for DatabaseErrorInformation — diesel's trait object.
        // We need this to construct DatabaseError variants for testing because
        // diesel doesn't expose a builder.
        struct StubDbErrorInfo(String);
        impl diesel::result::DatabaseErrorInformation for StubDbErrorInfo {
            fn message(&self) -> &str { &self.0 }
            fn details(&self) -> Option<&str> { None }
            fn hint(&self) -> Option<&str> { None }
            fn table_name(&self) -> Option<&str> { None }
            fn column_name(&self) -> Option<&str> { None }
            fn constraint_name(&self) -> Option<&str> { None }
            fn statement_position(&self) -> Option<i32> { None }
        }

        #[test]
        fn test_unique_violation_maps_to_conflict() {
            let info = Box::new(StubDbErrorInfo("login already taken".to_string()));
            let diesel_err = DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info);
            let app_err: AppError = diesel_err.into();
            match app_err {
                AppError::Conflict(msg) => assert_eq!(msg, "login already taken"),
                other => panic!("Expected Conflict, got {:?}", other),
            }
        }

        #[test]
        fn test_foreign_key_violation_maps_to_bad_request() {
            let info = Box::new(StubDbErrorInfo("fk constraint failed".to_string()));
            let diesel_err = DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, info);
            let app_err: AppError = diesel_err.into();
            match app_err {
                AppError::BadRequest(msg) => assert_eq!(msg, "fk constraint failed"),
                other => panic!("Expected BadRequest, got {:?}", other),
            }
        }

        #[test]
        fn test_other_database_error_kind_maps_to_internal() {
            // Any DatabaseError kind we don't explicitly handle falls through
            // to Internal. CheckViolation is a representative "other" kind.
            let info = Box::new(StubDbErrorInfo("check failed".to_string()));
            let diesel_err = DieselError::DatabaseError(DatabaseErrorKind::CheckViolation, info);
            let app_err: AppError = diesel_err.into();
            match app_err {
                AppError::Internal(msg) => assert_eq!(msg, "check failed"),
                other => panic!("Expected Internal, got {:?}", other),
            }
        }

        #[test]
        fn test_non_database_error_maps_to_internal() {
            // Catch-all arm: any other DieselError variant becomes Internal.
            let diesel_err = DieselError::RollbackTransaction;
            let app_err: AppError = diesel_err.into();
            match app_err {
                AppError::Internal(_) => {}
                other => panic!("Expected Internal, got {:?}", other),
            }
        }
    }
}
