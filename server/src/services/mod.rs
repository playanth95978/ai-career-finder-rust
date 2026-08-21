mod user_service;
pub use user_service::*;
mod auth_service;

pub use auth_service::{AuthService, Claims};






pub mod job_offer_service;
pub use job_offer_service::*;
pub mod candidate_profile_service;
pub use candidate_profile_service::*;
pub mod job_application_service;
pub use job_application_service::*;
pub mod user_preference_service;
pub use user_preference_service::*;
pub mod auto_apply_config_service;
pub use auto_apply_config_service::*;
pub mod radar_hit_service;
pub use radar_hit_service::*;
pub mod radar_state_service;
pub use radar_state_service::*;
pub mod conversation_service;
pub use conversation_service::*;
pub mod cv_resume_service;
pub use cv_resume_service::*;
pub mod cv_resume_version_service;
pub use cv_resume_version_service::*;
pub mod offer_positioning_service;
pub use offer_positioning_service::*;
pub mod offer_tailored_resume_service;
pub use offer_tailored_resume_service::*;
pub mod ai_service;
pub use ai_service::*;
pub mod embedding_service;
pub use embedding_service::*;
pub mod cv_extraction_service;
pub use cv_extraction_service::*;
pub mod mistral_ocr_service;
pub use mistral_ocr_service::*;

// jhipster-needle-add-entity-service - JHipster will add entity services here
