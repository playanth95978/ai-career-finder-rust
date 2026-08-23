use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

/// Application info response for JHipster UI
#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(rename_all = "kebab-case")]
pub struct InfoResponse {
    /// Display ribbon indicator on profiles
    pub display_ribbon_on_profiles: String,
    /// Currently active profiles
    #[serde(rename = "activeProfiles")]
    pub active_profiles: Vec<String>,
}

/// Reponse de `/management/health`, forme attendue par l'UI JHipster.
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// `UP` ou `DOWN`, seule valeur que le bandeau de statut interprete.
    pub status: String,
    /// Detail libre affiche a cote du statut.
    #[schema(value_type = Object)]
    pub details: serde_json::Value,
}

/// Management routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/info", get(info))
        .route("/health", get(health))
}

/// Get application info
///
/// Returns application metadata for JHipster UI ribbon and profile display.
#[utoipa::path(
    get,
    path = "/management/info",
    tag = "management",
    responses(
        (status = 200, description = "Application info", body = InfoResponse)
    )
)]
pub async fn info(State(state): State<AppState>) -> Json<InfoResponse> {
    // Use APP_PROFILE if set (Consul config profile), otherwise fall back to APP_ENV
    let profile = std::env::var("APP_PROFILE").unwrap_or_else(|_| state.config.app_env.clone());

    // Map common environment names to JHipster profile names
    let normalized = match profile.as_str() {
        "production" => "prod".to_string(),
        "development" => "dev".to_string(),
        other => other.to_string(),
    };

    // Determine active profiles based on environment
    let mut active_profiles = vec![normalized.clone()];
    // Include api-docs profile when not in production (enables Swagger UI link in Angular navbar)
    if normalized != "prod" {
        active_profiles.push("api-docs".to_string());
    }

    Json(InfoResponse {
        // Always "dev" — the Angular ribbon only shows when this value appears in activeProfiles.
        // In production, activeProfiles contains "prod" (not "dev"), so no ribbon is displayed.
        display_ribbon_on_profiles: "dev".to_string(),
        active_profiles,
    })
}

/// Health check
///
/// Sonde de disponibilite consommee par l'UI JHipster et par les orchestrateurs.
#[utoipa::path(
    get,
    path = "/management/health",
    tag = "management",
    responses(
        (status = 200, description = "Application disponible", body = HealthResponse)
    )
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "UP".to_string(),
        // `json!` et non `.parse()` : `serde_json::Value` implemente `FromStr` pour du JSON
        // valide, et une phrase nue n'en est pas — le `.unwrap()` paniquait a chaque appel.
        details: serde_json::json!({ "message": "Server app is started" }),
    })
}


// Track 1 Phase 1c (2026-05-11): integration tests for /management/info.
// The handler reads APP_PROFILE from std::env; tests use a module-local
// Mutex to serialize env mutations (same convention locked in Phase 1b).
// Gated to non-MongoDB because create_test_state currently requires the
// SQL pool path; mongo coverage lands in a later phase.
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_utils::create_test_state;
    use axum::Router;
    use axum_test::TestServer;
    use tokio::sync::Mutex;

    // Serializes env-touching info tests within this module. Other test
    // modules with their own ENV_LOCK remain independently parallel.
    // Uses tokio::sync::Mutex because the guard is held across .await
    // (the HTTP request) — std::sync::Mutex would risk blocking the
    // tokio worker thread (clippy::await_holding_lock).
    static ENV_LOCK: Mutex<()> = Mutex::const_new(());

    fn create_test_app() -> TestServer {
        let state = create_test_state();
        let app = Router::new()
            .nest("/management", routes())
            .with_state(state);
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_info_display_ribbon_is_always_dev() {
        let _g = ENV_LOCK.lock().await;
        std::env::remove_var("APP_PROFILE");
        let server = create_test_app();
        let response = server.get("/management/info").await;
        std::env::remove_var("APP_PROFILE");
        response.assert_status_ok();
        let body: InfoResponse = response.json();
        // Contract: this field is always "dev" so the Angular ribbon shows
        // only when "dev" appears in active_profiles.
        assert_eq!(body.display_ribbon_on_profiles, "dev");
    }

    #[tokio::test]
    async fn test_info_app_profile_env_overrides_config() {
        let _g = ENV_LOCK.lock().await;
        std::env::set_var("APP_PROFILE", "canary");
        let server = create_test_app();
        let response = server.get("/management/info").await;
        std::env::remove_var("APP_PROFILE");
        let body: InfoResponse = response.json();
        assert!(body.active_profiles.iter().any(|p| p == "canary"));
    }

    #[tokio::test]
    async fn test_info_maps_production_to_prod() {
        let _g = ENV_LOCK.lock().await;
        std::env::set_var("APP_PROFILE", "production");
        let server = create_test_app();
        let response = server.get("/management/info").await;
        std::env::remove_var("APP_PROFILE");
        let body: InfoResponse = response.json();
        assert!(body.active_profiles.contains(&"prod".to_string()));
        // Documented JHipster contract: NEVER include raw "production" in
        // active_profiles — clients key on "prod".
        assert!(!body.active_profiles.iter().any(|p| p == "production"));
    }

    #[tokio::test]
    async fn test_info_maps_development_to_dev() {
        let _g = ENV_LOCK.lock().await;
        std::env::set_var("APP_PROFILE", "development");
        let server = create_test_app();
        let response = server.get("/management/info").await;
        std::env::remove_var("APP_PROFILE");
        let body: InfoResponse = response.json();
        assert!(body.active_profiles.contains(&"dev".to_string()));
        assert!(!body.active_profiles.iter().any(|p| p == "development"));
    }

    #[tokio::test]
    async fn test_info_passes_through_arbitrary_profile() {
        // Only "production" and "development" get mapped; other values
        // pass through verbatim.
        let _g = ENV_LOCK.lock().await;
        std::env::set_var("APP_PROFILE", "staging");
        let server = create_test_app();
        let response = server.get("/management/info").await;
        std::env::remove_var("APP_PROFILE");
        let body: InfoResponse = response.json();
        assert!(body.active_profiles.contains(&"staging".to_string()));
    }

    #[tokio::test]
    async fn test_info_falls_back_to_config_app_env_when_no_env_var() {
        // When APP_PROFILE is unset, the handler reads state.config.app_env.
        // create_test_config() sets app_env to "test" — neither "production"
        // nor "development", so it passes through the `_ => other` arm verbatim.
        let _g = ENV_LOCK.lock().await;
        std::env::remove_var("APP_PROFILE");
        let server = create_test_app();
        let response = server.get("/management/info").await;
        let body: InfoResponse = response.json();
        assert!(body.active_profiles.contains(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_info_includes_api_docs_when_not_prod() {
        // Contract: enableSwaggerCodegen scaffolds add "api-docs" to
        // active_profiles in non-prod environments, enabling the Angular
        // navbar's Swagger UI link.
        let _g = ENV_LOCK.lock().await;
        std::env::set_var("APP_PROFILE", "development");
        let server = create_test_app();
        let response = server.get("/management/info").await;
        std::env::remove_var("APP_PROFILE");
        let body: InfoResponse = response.json();
        assert!(body.active_profiles.contains(&"api-docs".to_string()));
    }

    #[tokio::test]
    async fn test_info_excludes_api_docs_in_prod() {
        // Mirror contract: prod must NOT advertise the Swagger UI link.
        let _g = ENV_LOCK.lock().await;
        std::env::set_var("APP_PROFILE", "production");
        let server = create_test_app();
        let response = server.get("/management/info").await;
        std::env::remove_var("APP_PROFILE");
        let body: InfoResponse = response.json();
        assert!(!body.active_profiles.iter().any(|p| p == "api-docs"));
    }
}
