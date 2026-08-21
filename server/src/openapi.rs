//! OpenAPI/Swagger documentation configuration
//!
//! This module configures the OpenAPI specification for API documentation.
//! Available documentation endpoints:
//! - Swagger UI: /swagger-ui (served from static files, JHipster's custom UI with auth integration)
//! - Scalar UI: /scalar (alternative API documentation viewer)
//! - OpenAPI JSON: /v3/api-docs (Spring Boot compatible endpoint)

use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

use crate::dto::*;
use crate::errors::ErrorResponse;
use crate::handlers;

/// API Documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "jobSearchRust API",
        description = "jobSearchRust REST API documentation",
        version = "1.0.0",
        license(name = "Apache 2.0", url = "https://www.apache.org/licenses/LICENSE-2.0"),
        contact(
            name = "jobSearchRust Team",
            email = "team@job-search-rust.com"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server")
    ),
    paths(
        // Health endpoint
        handlers::health::health_check,
        // Management endpoint
        handlers::management::info,
        // Account endpoints
        handlers::account::get_account,
        handlers::account::save_account,
        handlers::account::authenticate,
        handlers::account::change_password,
        // User management endpoints
        handlers::user::get_all_users,
        handlers::user::get_user,
        handlers::user::create_user,
        handlers::user::update_user_from_body,
        handlers::user::delete_user,
        handlers::user::get_authorities,
        handlers::job_offer::get_all,
                handlers::job_offer::get_one,
                handlers::job_offer::create,
                handlers::job_offer::update,
                handlers::job_offer::remove,
        handlers::candidate_profile::get_all,
                handlers::candidate_profile::get_one,
                handlers::candidate_profile::create,
                handlers::candidate_profile::update,
                handlers::candidate_profile::remove,
        handlers::job_application::get_all,
                handlers::job_application::get_one,
                handlers::job_application::create,
                handlers::job_application::update,
                handlers::job_application::remove,
        handlers::user_preference::get_all,
                handlers::user_preference::get_one,
                handlers::user_preference::create,
                handlers::user_preference::update,
                handlers::user_preference::remove,
        handlers::auto_apply_config::get_all,
                handlers::auto_apply_config::get_one,
                handlers::auto_apply_config::create,
                handlers::auto_apply_config::update,
                handlers::auto_apply_config::remove,
        handlers::radar_hit::get_all,
                handlers::radar_hit::get_one,
                handlers::radar_hit::create,
                handlers::radar_hit::update,
                handlers::radar_hit::remove,
        handlers::radar_state::get_all,
                handlers::radar_state::get_one,
                handlers::radar_state::create,
                handlers::radar_state::update,
                handlers::radar_state::remove,
        handlers::conversation::get_all,
                handlers::conversation::get_one,
                handlers::conversation::create,
                handlers::conversation::update,
                handlers::conversation::remove,
        handlers::cv_resume::get_all,
                handlers::cv_resume::get_one,
                handlers::cv_resume::create,
                handlers::cv_resume::update,
                handlers::cv_resume::remove,
        handlers::cv_resume_version::get_all,
                handlers::cv_resume_version::get_one,
                handlers::cv_resume_version::create,
                handlers::cv_resume_version::update,
                handlers::cv_resume_version::remove,
        handlers::offer_positioning::get_all,
                handlers::offer_positioning::get_one,
                handlers::offer_positioning::create,
                handlers::offer_positioning::update,
                handlers::offer_positioning::remove,
        handlers::offer_tailored_resume::get_all,
                handlers::offer_tailored_resume::get_one,
                handlers::offer_tailored_resume::create,
                handlers::offer_tailored_resume::update,
                handlers::offer_tailored_resume::remove,
        // jhipster-needle-add-openapi-path - JHipster will add OpenAPI paths here
    ),
    components(
        schemas(
            // Common schemas
            ErrorResponse,
            PageRequest,
            // User schemas
            UserDto,
            CreateUserDto,
            UpdateUserDto,
            handlers::account::SaveAccountRequest,
            handlers::account::LoginRequest,
            handlers::account::LoginResponse,
            handlers::account::ChangePasswordRequest,
            // Management schemas
            handlers::management::InfoResponse,
            // Health schemas
            HealthStatus,
            HealthComponent,
            HealthResponse,
            JobOfferDto,
                        CreateJobOfferDto,
                        UpdateJobOfferDto,
            CandidateProfileDto,
                        CreateCandidateProfileDto,
                        UpdateCandidateProfileDto,
            JobApplicationDto,
                        CreateJobApplicationDto,
                        UpdateJobApplicationDto,
            UserPreferenceDto,
                        CreateUserPreferenceDto,
                        UpdateUserPreferenceDto,
            AutoApplyConfigDto,
                        CreateAutoApplyConfigDto,
                        UpdateAutoApplyConfigDto,
            RadarHitDto,
                        CreateRadarHitDto,
                        UpdateRadarHitDto,
            RadarStateDto,
                        CreateRadarStateDto,
                        UpdateRadarStateDto,
            ConversationDto,
                        CreateConversationDto,
                        UpdateConversationDto,
            CvResumeDto,
                        CreateCvResumeDto,
                        UpdateCvResumeDto,
            CvResumeVersionDto,
                        CreateCvResumeVersionDto,
                        UpdateCvResumeVersionDto,
            OfferPositioningDto,
                        CreateOfferPositioningDto,
                        UpdateOfferPositioningDto,
            OfferTailoredResumeDto,
                        CreateOfferTailoredResumeDto,
                        UpdateOfferTailoredResumeDto,
            // jhipster-needle-add-openapi-schema - JHipster will add OpenAPI schemas here
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "management", description = "Management/actuator endpoints"),
        (name = "account", description = "Account management endpoints"),
        (name = "authentication", description = "Authentication endpoints"),
        (name = "user-management", description = "User administration endpoints"),
        (name = "job-offers", description = "JobOffer management endpoints"),
        (name = "candidate-profiles", description = "CandidateProfile management endpoints"),
        (name = "job-applications", description = "JobApplication management endpoints"),
        (name = "user-preferences", description = "UserPreference management endpoints"),
        (name = "auto-apply-configs", description = "AutoApplyConfig management endpoints"),
        (name = "radar-hits", description = "RadarHit management endpoints"),
        (name = "radar-states", description = "RadarState management endpoints"),
        (name = "conversations", description = "Conversation management endpoints"),
        (name = "cv-resumes", description = "CvResume management endpoints"),
        (name = "cv-resume-versions", description = "CvResumeVersion management endpoints"),
        (name = "offer-positionings", description = "OfferPositioning management endpoints"),
        (name = "offer-tailored-resumes", description = "OfferTailoredResume management endpoints"),
        // jhipster-needle-add-openapi-tag - JHipster will add OpenAPI tags here
    )
)]
pub struct ApiDoc;

/// Security scheme modifier for JWT Bearer authentication
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Enter your JWT token"))
                    .build(),
            ),
        );
    }
}

// Helper structs for documentation
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Health check status
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthStatus {
    /// Overall status (UP, DOWN)
    pub status: String,
}

/// Health component status
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthComponent {
    /// Component name
    pub name: String,
    /// Component status
    pub status: String,
    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Overall status
    pub status: String,
    /// Individual component statuses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<HealthComponent>>,
}

// Track 1 Phase 1d (2026-05-11): assert ApiDoc::openapi() produces a doc
// with the expected paths registered and that SecurityAddon runs (its
// side effect is adding the bearer_auth scheme to components).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_doc_registers_known_paths() {
        let doc = ApiDoc::openapi();
        // Pin at least one known endpoint so this test fails loudly if the
        // utoipa attribute macro silently drops paths during template churn.
        assert!(
            doc.paths.paths.contains_key("/api/health"),
            "/api/health missing from openapi paths; found: {:?}",
            doc.paths.paths.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_api_doc_security_addon_registers_bearer_auth_scheme() {
        // SecurityAddon's modify() is invoked as part of ApiDoc::openapi().
        // We verify the side effect (bearer_auth scheme present) rather than
        // calling modify() directly, which would couple the test to internals.
        let doc = ApiDoc::openapi();
        let components = doc.components.as_ref().expect("components should be present");
        assert!(
            components.security_schemes.contains_key("bearer_auth"),
            "bearer_auth scheme missing; found: {:?}",
            components.security_schemes.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_api_doc_info_metadata_populated() {
        // The info section is templated from EJS — assert non-empty so
        // template typos that drop the macro args get caught.
        let doc = ApiDoc::openapi();
        assert!(!doc.info.title.is_empty());
        assert!(!doc.info.version.is_empty());
    }
}
