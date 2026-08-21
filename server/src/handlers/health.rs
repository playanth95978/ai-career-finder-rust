use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// Overall health status (UP or DOWN)
    pub status: String,
    /// Component-level health status
    pub components: HealthComponents,
}

#[derive(Serialize, ToSchema)]
pub struct HealthComponents {
    /// Database health status
    pub db: ComponentHealth,
}

#[derive(Serialize, ToSchema)]
pub struct ComponentHealth {
    /// Component status (UP or DOWN)
    pub status: String,
}

/// Health check routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health_check))
        .route("/liveness", get(liveness))
        .route("/readiness", get(readiness))
}

/// Main health check endpoint
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "Health check successful", body = HealthResponse)
    )
)]
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let db_status = check_database(&state).await;

    Json(HealthResponse {
        status: if db_status { "UP" } else { "DOWN" }.to_string(),
        components: HealthComponents {
            db: ComponentHealth {
                status: if db_status { "UP" } else { "DOWN" }.to_string(),
            },
        },
    })
}

/// Kubernetes liveness probe
async fn liveness() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "UP" }))
}

/// Kubernetes readiness probe
async fn readiness(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db_status = check_database(&state).await;
    Json(serde_json::json!({
        "status": if db_status { "UP" } else { "DOWN" }
    }))
}

async fn check_database(state: &AppState) -> bool {
    state.pool.get().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "UP".to_string(),
            components: HealthComponents {
                db: ComponentHealth {
                    status: "UP".to_string(),
                },
            },
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"UP\""));
        assert!(json.contains("\"components\""));
        assert!(json.contains("\"db\""));
    }

    #[test]
    fn test_health_response_down_status() {
        let response = HealthResponse {
            status: "DOWN".to_string(),
            components: HealthComponents {
                db: ComponentHealth {
                    status: "DOWN".to_string(),
                },
            },
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"DOWN\""));
    }

    #[test]
    fn test_component_health_serialization() {
        let component = ComponentHealth {
            status: "UP".to_string(),
        };
        let json = serde_json::to_string(&component).unwrap();
        assert_eq!(json, "{\"status\":\"UP\"}");
    }

    #[test]
    fn test_health_components_serialization() {
        let components = HealthComponents {
            db: ComponentHealth {
                status: "UP".to_string(),
            },
        };
        let json = serde_json::to_string(&components).unwrap();
        assert!(json.contains("\"db\""));
        assert!(json.contains("\"status\":\"UP\""));
    }
}

// Track 1 Phase 1c (2026-05-11): integration tests for the health routes.
// Gated to non-MongoDB because `create_test_pool()` requires a DATABASE_URL
// to a real Postgres/MySQL/SQLite instance (CI provisions one via the
// `coverage` job's postgres service container). The mongo path uses a
// different test fixture not built out yet — Phase 1+ work.
#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::Router;
    use axum_test::TestServer;
    use crate::test_utils::create_test_state;

    fn create_test_app() -> TestServer {
        let state = create_test_state();
        let app = Router::new()
            .nest("/health", routes())
            .with_state(state);
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_health_check_returns_up_with_working_pool() {
        // axum 0.7 normalizes nested-route trailing slashes — the route is
        // mounted at "/health", not "/health/".
        let server = create_test_app();
        let response = server.get("/health").await;
        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "UP");
        assert_eq!(body["components"]["db"]["status"], "UP");
    }

    #[tokio::test]
    async fn test_health_check_top_level_status_matches_db_component() {
        // Documented contract: the overall status reflects the DB component.
        // If the DB is up, top-level is "UP"; if down, top-level is "DOWN".
        let server = create_test_app();
        let response = server.get("/health").await;
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], body["components"]["db"]["status"]);
    }

    #[tokio::test]
    async fn test_liveness_always_returns_up_regardless_of_state() {
        // Liveness ignores DB state — it's a "process is alive" probe for
        // Kubernetes. Even with a broken pool, this must return UP. Here we
        // assert the happy path; the contract is "always UP" by construction.
        let server = create_test_app();
        let response = server.get("/health/liveness").await;
        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "UP");
    }

    #[tokio::test]
    async fn test_readiness_returns_up_with_working_pool() {
        let server = create_test_app();
        let response = server.get("/health/readiness").await;
        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "UP");
    }

    #[tokio::test]
    async fn test_health_response_has_expected_json_shape() {
        let server = create_test_app();
        let response = server.get("/health").await;
        let text = response.text();
        // Pin the field names; tooling (Spring Boot Actuator clients,
        // K8s probes) keys on these exact strings.
        assert!(text.contains("\"status\":"));
        assert!(text.contains("\"components\":"));
        assert!(text.contains("\"db\":"));
    }
}
