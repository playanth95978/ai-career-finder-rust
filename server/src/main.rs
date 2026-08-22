use axum::{http::{header, HeaderValue, Method}, middleware, routing::get, Router};
use axum::Json;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use job_search_rust::config::AppConfig;
use job_search_rust::openapi::ApiDoc;
use job_search_rust::db::connection::establish_connection_pool;
use job_search_rust::handlers;
use job_search_rust::middleware::auth::auth_middleware;
use job_search_rust::AppState;

#[tokio::main]
async fn main() {
    // Load .env file first so RUST_LOG is available for tracing initialization
    dotenvy::dotenv().ok();

    // Initialize tracing (reads RUST_LOG from environment)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,job_search_rust=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::from_env();

    // Defense-in-depth: refuse to start if JWT_SECRET is set to a known-default
    // sentinel value. The docker-entrypoint.sh script catches this earlier in the
    // boot path for `docker run` workflows, but K8s static-manifest users (kubectl
    // apply -f without modifying app-secret.yml) can still get here with the
    // sentinel literal injected via envFrom: secretRef.
    if job_search_rust::config::sentinels::is_sentinel(&config.jwt_secret) {
        eprintln!(
            "FATAL: JWT_SECRET is set to a known-default sentinel value. \
             Refusing to start. See RELEASE_NOTES.md for migration guidance."
        );
        std::process::exit(1);
    }

    // Establish database connection pool
    let pool = establish_connection_pool(&config.database_url);

    // Run migrations
    job_search_rust::db::connection::run_migrations(&pool);

    // Build CORS layer.
    // Development: permissive (Any) so local frontends on arbitrary ports work.
    // Production: allowlist origins from CORS_ALLOWED_ORIGINS (comma-separated),
    // restrict methods/headers, refuse to start if the env var is missing or empty.
    let cors = if config.is_production() {
        let raw = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| {
            eprintln!("FATAL: CORS_ALLOWED_ORIGINS must be set in production. Refusing to start.");
            std::process::exit(1);
        });
        let allowed: Vec<HeaderValue> = raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<HeaderValue>().ok())
            .collect();
        if allowed.is_empty() {
            eprintln!("FATAL: CORS_ALLOWED_ORIGINS produced an empty allowlist (raw value: {:?}). Refusing to start.", raw);
            std::process::exit(1);
        }
        CorsLayer::new()
            .allow_origin(allowed)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
    } else {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Poller d'embedding des offres : l'ingestion les marque PENDING, ce poller ecrit leur
    // vecteur en tache de fond. Sans lui, `/jobs/indexed-count` resterait a zero et la recherche
    // semantique se replierait indefiniment sur le lexical.
    job_search_rust::services::job_offer_embedding_service::JobOfferEmbeddingService::spawn_poller(
        pool.clone(),
    );

    // Build application state
    let state = AppState::new(pool, config.clone());

    // Build application routes
    let mut app = Router::new()
        .nest("/api", api_routes(state.clone()))
        .nest("/management", handlers::management::routes())
        // OpenAPI documentation endpoints
        // Note: /swagger-ui/* is served from static files (JHipster's custom Swagger UI)
        // Scalar provides an alternative API documentation viewer
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        // Spring Boot compatible endpoint for JHipster Angular UI
        .route("/v3/api-docs", get(openapi_json))
        .layer(TraceLayer::new_for_http())
        .layer(cors);


    // Add static file serving if configured
    if config.serve_static_files {
        let static_dir = config.static_files_dir.clone()
            .unwrap_or_else(|| "./static".to_string());

        tracing::info!("Serving static files from: {}", static_dir);

        // Serve static files with SPA fallback for Angular routes
        let static_service = ServeDir::new(&static_dir)
            .fallback(handlers::static_files::spa_fallback_handler(static_dir.clone()));

        app = app.fallback_service(static_service);
    } else {
        // Simple welcome message when not serving static files
        app = app.route("/", get(|| async { "Welcome to jobSearchRust!" }));
    }

    let app = app.with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.app_port));
    tracing::info!("Starting {} server on {}", "jobSearchRust", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn api_routes(state: AppState) -> Router<AppState> {
    // Routes that require authentication
    let protected_routes = Router::new()
        .nest("/users", handlers::user::routes())
        .nest("/admin/users", handlers::user::admin_routes())
        .nest("/authorities", handlers::user::authority_routes())
        .nest("/account", handlers::account::routes())
        .nest("/job-offers", handlers::job_offer::routes())
        .nest("/candidate-profiles", handlers::candidate_profile::routes())
        .nest("/job-applications", handlers::job_application::routes())
        .nest("/user-preferences", handlers::user_preference::routes())
        .nest("/auto-apply-configs", handlers::auto_apply_config::routes())
        .nest("/radar-hits", handlers::radar_hit::routes())
        .nest("/radar-states", handlers::radar_state::routes())
        .nest("/conversations", handlers::conversation::routes())
        .nest("/cv-resumes", handlers::cv_resume::routes())
        .nest("/cv-resume-versions", handlers::cv_resume_version::routes())
        .nest("/offer-positionings", handlers::offer_positioning::routes())
        .nest("/offer-tailored-resumes", handlers::offer_tailored_resume::routes())
        .nest("/mistral-ocr", handlers::mistral_ocr::routes())
        .nest("/job-copilot/cv", handlers::job_copilot_cv::routes())
        .nest("/job-copilot/preferences", handlers::job_copilot_preference::routes())
        .nest("/job-copilot/radar", handlers::job_copilot_radar::routes())
        .nest("/job-copilot/jobs", handlers::job_copilot_search::routes())
        .nest("/job-copilot/applications", handlers::job_copilot_application::routes())
        .nest("/cv-builder", handlers::cv_builder::routes())
        .nest("/chat-history", handlers::chat_history::routes())
        .nest("/job-copilot/assistant", handlers::job_copilot_assistant::routes())
        // jhipster-needle-add-entity-route - JHipster will add entity routes here
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .nest("/health", handlers::health::routes())
        .nest("/ai", handlers::ai::routes())
        // Registration routes (public for JWT authentication)
        .merge(handlers::account::public_routes())
        .nest("/authenticate", handlers::account::auth_routes());

    Router::new()
        .merge(protected_routes)
        .merge(public_routes)
}

/// Serve OpenAPI JSON at /v3/api-docs for JHipster Angular UI compatibility
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
