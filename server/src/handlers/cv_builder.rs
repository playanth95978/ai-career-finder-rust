//! CV Builder (`/api/cv-builder`) : persistance et versionnage du CV editable.
//!
//! Portee volontairement limitee a la persistance. Les endpoints assistes par le modele de la
//! version Java (`/generate`, `/match`, `/review`, `/rewrite`, `/translate`) et l'export PDF ne
//! sont pas portes ici : l'editeur est utilisable sans eux, ils ne le sont pas sans elle.

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Extension, Json, Router,
};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::candidate_profile;
use crate::dto::job_copilot_dto::{ResumeDto, ResumeVersionDto, SaveResumeDto};
use crate::dto::CandidateProfileDto;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::{CandidateProfile, RoleType};
use crate::services::cv_builder_service::CvBuilderService;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/resume", get(get_resume).put(save_resume))
        .route("/resume/versions", get(list_versions).post(create_version))
        .route("/resume/versions/:id", get(get_version))
        .route("/resume/versions/:id/restore", post(restore_version))
        .route("/resume/candidate-profile", get(get_candidate_profile))
}

/// CV courant de l'utilisateur, 404 s'il n'en a pas encore.
#[utoipa::path(
    get,
    path = "/api/cv-builder/resume",
    tag = "cv-builder",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "CV courant", body = ResumeDto),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Aucun CV enregistre")
    )
)]
pub async fn get_resume(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<ResumeDto>, AppError> {
    let mut conn = require_user(&state, &auth)?;

    CvBuilderService::find_for_user(&mut conn, &auth.login)?
        .map(|r| Json(ResumeDto::from(r)))
        .ok_or_else(|| AppError::NotFound("No resume for this user".into()))
}

/// Auto-sauvegarde du CV. Aucun instantane n'est cree : l'historique reste pilote par
/// `POST /resume/versions`, sinon chaque frappe clavier y ajouterait une entree.
#[utoipa::path(
    put,
    path = "/api/cv-builder/resume",
    tag = "cv-builder",
    security(("bearer_auth" = [])),
    request_body = SaveResumeDto,
    responses(
        (status = 200, description = "CV enregistre", body = ResumeDto),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn save_resume(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(dto): Json<SaveResumeDto>,
) -> Result<Json<ResumeDto>, AppError> {
    let mut conn = require_user(&state, &auth)?;
    let saved = CvBuilderService::save(&mut conn, &auth.login, dto)?;
    Ok(Json(ResumeDto::from(saved)))
}

/// Fige l'etat courant du CV dans l'historique.
#[utoipa::path(
    post,
    path = "/api/cv-builder/resume/versions",
    tag = "cv-builder",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Version creee", body = ResumeVersionDto),
        (status = 400, description = "Aucun CV a versionner"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create_version(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<ResumeVersionDto>, AppError> {
    let mut conn = require_user(&state, &auth)?;
    let version = CvBuilderService::create_version(&mut conn, &auth.login)?;
    Ok(Json(ResumeVersionDto::from(version)))
}

/// Historique des versions, de la plus recente a la plus ancienne.
#[utoipa::path(
    get,
    path = "/api/cv-builder/resume/versions",
    tag = "cv-builder",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Versions du CV", body = Vec<ResumeVersionDto>),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn list_versions(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<ResumeVersionDto>>, AppError> {
    let mut conn = require_user(&state, &auth)?;
    let versions = CvBuilderService::list_versions(&mut conn, &auth.login)?;
    Ok(Json(versions.into_iter().map(ResumeVersionDto::from).collect()))
}

/// Contenu d'une version passee, renvoye sous la meme forme que le CV courant.
#[utoipa::path(
    get,
    path = "/api/cv-builder/resume/versions/{id}",
    tag = "cv-builder",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Identifiant de la version")),
    responses(
        (status = 200, description = "Contenu de la version", body = ResumeDto),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Version inconnue")
    )
)]
pub async fn get_version(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResumeDto>, AppError> {
    let mut conn = require_user(&state, &auth)?;

    CvBuilderService::find_version(&mut conn, &auth.login, id)?
        .map(|v| {
            Json(ResumeDto {
                id: v.id,
                title: v.title,
                template: v.template,
                data: v.data,
                version: v.version_number,
                updated_at: v.created_at.map(|d| d.to_string()),
            })
        })
        .ok_or_else(|| AppError::NotFound("Resume version not found".into()))
}

/// Restaure une version dans le CV courant, apres avoir archive l'etat en cours.
#[utoipa::path(
    post,
    path = "/api/cv-builder/resume/versions/{id}/restore",
    tag = "cv-builder",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Identifiant de la version a restaurer")),
    responses(
        (status = 200, description = "CV restaure", body = ResumeDto),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Version inconnue")
    )
)]
pub async fn restore_version(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResumeDto>, AppError> {
    let mut conn = require_user(&state, &auth)?;

    CvBuilderService::restore_version(&mut conn, &auth.login, id)?
        .map(|r| Json(ResumeDto::from(r)))
        .ok_or_else(|| AppError::NotFound("Resume version not found".into()))
}

/// Profil candidat extrait du CV importe, que le CV Builder utilise comme point de depart.
#[utoipa::path(
    get,
    path = "/api/cv-builder/resume/candidate-profile",
    tag = "cv-builder",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Profil candidat", body = CandidateProfileDto),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Aucun profil : importer un CV d'abord")
    )
)]
pub async fn get_candidate_profile(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<CandidateProfileDto>, AppError> {
    let mut conn = require_user(&state, &auth)?;

    let found: Option<CandidateProfile> = candidate_profile::table
        .filter(candidate_profile::user_id.eq(&auth.login))
        .order(candidate_profile::created_at.desc().nulls_last())
        .select(CandidateProfile::as_select())
        .first(&mut conn)
        .optional()?;

    found
        .map(|p| Json(CandidateProfileDto::from(p)))
        .ok_or_else(|| AppError::NotFound("No CV profile for this user".into()))
}

/// Verifie le role puis prend une connexion. Regroupe les deux gardes repetees par chaque
/// endpoint de ce module.
fn require_user(
    state: &AppState,
    auth: &AuthUser,
) -> Result<crate::db::connection::DbConnection, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }
    state.pool.get().map_err(|e| AppError::Internal(e.to_string()))
}
