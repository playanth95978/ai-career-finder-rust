//! Endpoints CV du Job Copilot, alignes sur les URL que le front Angular appelle deja
//! (`/api/job-copilot/cv/upload`, `/api/job-copilot/cv/profile`).

use axum::{
    extract::{Multipart, State},
    routing::{get, post},
    Extension, Json, Router,
};
use diesel::prelude::*;

use crate::db::schema::candidate_profile;
use crate::dto::CandidateProfileDto;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::{CandidateProfile, RoleType};
use crate::services::cv_ingestion_service::CvIngestionService;
use crate::AppState;

/// Taille maximale, alignee sur `spring.servlet.multipart.max-file-size` (50 Mo).
const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload))
        .route("/profile", get(profile))
}

/// Ingere un CV (PDF ou DOCX) et renvoie le profil candidat extrait.
#[utoipa::path(
    post,
    path = "/api/job-copilot/cv/upload",
    tag = "job-copilot-cv",
    request_body(content = String, description = "multipart/form-data avec un champ `file`", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Profil candidat extrait du CV", body = CandidateProfileDto),
        (status = 400, description = "Fichier manquant, vide ou illisible"),
        (status = 500, description = "Erreur OCR, LLM ou base de donnees")
    )
)]
pub async fn upload(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<Json<CandidateProfileDto>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart payload: {e}")))?
    {
        if field.name() != Some("file") {
            continue;
        }
        filename = field.file_name().map(str::to_owned);
        content_type = field.content_type().map(str::to_owned);
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Could not read uploaded file: {e}")))?;
        if data.len() > MAX_UPLOAD_BYTES {
            return Err(AppError::BadRequest(format!(
                "File too large: {} bytes (max {MAX_UPLOAD_BYTES})",
                data.len()
            )));
        }
        bytes = Some(data.to_vec());
        break;
    }

    let bytes = bytes.ok_or_else(|| AppError::BadRequest("Missing 'file' part in the request".into()))?;
    if bytes.is_empty() {
        return Err(AppError::BadRequest("Uploaded file is empty".into()));
    }

    // La connexion est prise apres l'upload : l'OCR et le LLM sont lents, inutile d'immobiliser
    // une connexion du pool pendant ces appels reseau.
    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let profile = CvIngestionService::ingest(
        &mut conn,
        &bytes,
        filename.as_deref(),
        content_type.as_deref(),
        &auth.login,
    )
    .await?;

    Ok(Json(CandidateProfileDto::from(profile)))
}

/// Renvoie le profil candidat courant, ou 404 si aucun CV n'a encore ete importe.
#[utoipa::path(
    get,
    path = "/api/job-copilot/cv/profile",
    tag = "job-copilot-cv",
    responses(
        (status = 200, description = "Profil candidat courant", body = CandidateProfileDto),
        (status = 404, description = "Aucun profil : importer un CV d'abord")
    )
)]
pub async fn profile(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<CandidateProfileDto>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let found: Option<CandidateProfile> = candidate_profile::table
        .filter(candidate_profile::user_id.eq(&auth.login))
        .order(candidate_profile::created_at.desc())
        .select(CandidateProfile::as_select())
        .first(&mut conn)
        .optional()?;

    found
        .map(|p| Json(CandidateProfileDto::from(p)))
        .ok_or_else(|| AppError::NotFound("No CV profile for this user".into()))
}
