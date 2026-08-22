//! Suivi de candidatures (`/api/job-copilot/applications`).

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put},
    Extension, Json, Router,
};
use chrono::Utc;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::schema::{candidate_profile, job_application, job_offer};
use crate::db::DbConnection;
use crate::dto::job_copilot_dto::JobApplicationViewDto;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::{
    CandidateProfile, JobApplication, JobOffer, NewJobApplication, RoleType,
};
use crate::services::cover_letter_service::CoverLetterService;
use crate::AppState;

/// Statuts de candidature, miroir de l'enum Java `ApplicationStatus`. Le front les type en union
/// de chaines : un statut hors de cette liste casserait son affichage, on le refuse donc.
const STATUSES: [&str; 7] = [
    "DRAFT",
    "APPLIED",
    "INTERVIEW",
    "REJECTED",
    "GHOSTED",
    "OFFER",
    "WITHDRAWN",
];

const DEFAULT_STATUS: &str = "DRAFT";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_applications))
        .route("/stats", get(get_stats))
        .route("/create", post(create_application))
        .route("/:id/status", put(update_status))
        // Meme segment dynamique pour les deux verbes : axum exige un nom de parametre unique
        // par chemin. Le PUT recoit un identifiant de candidature, le POST un identifiant
        // d'offre — c'est la methode qui distingue les deux, comme cote Java.
        .route(
            "/:id/cover-letter",
            put(update_cover_letter).post(generate_cover_letter),
        )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationParams {
    pub job_offer_id: Uuid,
    /// Absent = true, comme la valeur par defaut du controleur Java.
    pub generate_cover_letter: Option<bool>,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusParam {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct LanguageParam {
    pub language: Option<String>,
}

/// Candidatures de l'utilisateur, les plus recentes d'abord.
#[utoipa::path(
    get,
    path = "/api/job-copilot/applications",
    tag = "job-copilot-applications",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Candidatures", body = Vec<JobApplicationViewDto>),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_applications(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<JobApplicationViewDto>>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    // Jointure a gauche : une candidature dont l'offre a ete purgee reste listee, avec son statut
    // et sa lettre, plutot que de disparaitre du suivi de l'utilisateur.
    let rows: Vec<(JobApplication, Option<JobOffer>)> = job_application::table
        .left_join(job_offer::table.on(job_offer::id.nullable().eq(job_application::jobOffer_id)))
        .filter(job_application::user_id.eq(&auth.login))
        .order(job_application::created_at.desc().nulls_last())
        .select((JobApplication::as_select(), Option::<JobOffer>::as_select()))
        .load(&mut conn)?;

    Ok(Json(
        rows.iter()
            .map(|(application, offer)| {
                JobApplicationViewDto::new(application, offer.as_ref(), DEFAULT_STATUS)
            })
            .collect(),
    ))
}

/// Repartition des candidatures par statut, pour le tableau de bord.
///
/// Les statuts sans candidature sont presents a zero : le front affiche une colonne par statut et
/// afficherait des trous si le serveur ne renvoyait que les statuts utilises.
#[utoipa::path(
    get,
    path = "/api/job-copilot/applications/stats",
    tag = "job-copilot-applications",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Nombre de candidatures par statut"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_stats(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<HashMap<String, i64>>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let rows: Vec<(Option<String>, i64)> = job_application::table
        .filter(job_application::user_id.eq(&auth.login))
        .group_by(job_application::status)
        .select((job_application::status, diesel::dsl::count_star()))
        .load(&mut conn)?;

    let mut stats: HashMap<String, i64> =
        STATUSES.iter().map(|s| (s.to_string(), 0)).collect();
    for (status, count) in rows {
        // Une candidature au statut NULL est comptee comme brouillon plutot que perdue.
        let key = status.unwrap_or_else(|| DEFAULT_STATUS.to_string());
        *stats.entry(key).or_insert(0) += count;
    }

    Ok(Json(stats))
}

/// Cree une candidature pour une offre, avec generation optionnelle de la lettre.
#[utoipa::path(
    post,
    path = "/api/job-copilot/applications/create",
    tag = "job-copilot-applications",
    security(("bearer_auth" = [])),
    params(
        ("jobOfferId" = Uuid, Query, description = "Offre visee"),
        ("generateCoverLetter" = Option<bool>, Query, description = "Generer la lettre (defaut: true)"),
        ("language" = Option<String>, Query, description = "Langue de la lettre")
    ),
    responses(
        (status = 200, description = "Candidature creee", body = JobApplicationViewDto),
        (status = 400, description = "Offre inconnue ou profil candidat absent"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create_application(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(params): Query<CreateApplicationParams>,
) -> Result<Json<JobApplicationViewDto>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let (profile, offer) = {
        let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
        load_profile_and_offer(&mut conn, &auth.login, params.job_offer_id)?
    };

    // La lettre est generee hors connexion au pool : l'appel au modele dure plusieurs secondes,
    // inutile d'immobiliser une connexion pendant ce temps.
    let cover_letter = if params.generate_cover_letter.unwrap_or(true) {
        match CoverLetterService::generate(&profile, &offer, params.language.as_deref()).await {
            Ok(letter) => Some(letter),
            // Une lettre non generee ne doit pas empecher d'enregistrer la candidature :
            // l'utilisateur pourra la relancer depuis l'ecran de suivi.
            Err(e) => {
                tracing::warn!(offer = %offer.id, error = %e, "Generation de lettre echouee, candidature creee sans lettre");
                None
            }
        }
    } else {
        None
    };

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let now = Utc::now().naive_utc();

    // Une candidature existante est renvoyee telle quelle : le front appelle `create` depuis
    // plusieurs ecrans, et un doublon ferait apparaitre deux lignes pour la meme offre.
    if let Some(existing) = find_for_offer(&mut conn, &auth.login, offer.id)? {
        return Ok(Json(JobApplicationViewDto::new(&existing, Some(&offer), DEFAULT_STATUS)));
    }

    let new_application = NewJobApplication {
        user_id: auth.login.clone(),
        status: Some(DEFAULT_STATUS.to_string()),
        cover_letter,
        notes: None,
        match_score: None,
        created_at: Some(now),
        updated_at: Some(now),
        applied_at: None,
        jobOffer_id: Some(offer.id),
        candidateProfile_id: Some(profile.id),
        created_by: Some(auth.login.clone()),
        created_date: Some(now),
        last_modified_by: Some(auth.login.clone()),
        last_modified_date: Some(now),
    };

    let saved: JobApplication = diesel::insert_into(job_application::table)
        .values(&new_application)
        .returning(JobApplication::as_returning())
        .get_result(&mut conn)?;

    Ok(Json(JobApplicationViewDto::new(&saved, Some(&offer), DEFAULT_STATUS)))
}

/// Change le statut d'une candidature.
#[utoipa::path(
    put,
    path = "/api/job-copilot/applications/{id}/status",
    tag = "job-copilot-applications",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Identifiant de la candidature"),
        ("status" = String, Query, description = "Nouveau statut")
    ),
    responses(
        (status = 200, description = "Statut mis a jour", body = JobApplicationViewDto),
        (status = 400, description = "Statut inconnu"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Candidature inconnue")
    )
)]
pub async fn update_status(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(params): Query<StatusParam>,
) -> Result<Json<JobApplicationViewDto>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let status = normalize_status(&params.status)?;
    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let now = Utc::now().naive_utc();

    let existing = find_owned(&mut conn, &auth.login, id)?;

    // `applied_at` est horodate au premier passage a APPLIED et jamais reecrit ensuite : c'est la
    // date d'envoi reelle, pas celle du dernier changement de statut.
    let applied_at = match (status, existing.applied_at) {
        ("APPLIED", None) => Some(now),
        (_, previous) => previous,
    };

    let updated: JobApplication = diesel::update(job_application::table.find(id))
        .set((
            job_application::status.eq(status.to_string()),
            job_application::applied_at.eq(applied_at),
            job_application::updated_at.eq(now),
            job_application::last_modified_by.eq(auth.login.clone()),
            job_application::last_modified_date.eq(now),
        ))
        .returning(JobApplication::as_returning())
        .get_result(&mut conn)?;

    let offer = related_offer(&mut conn, &updated)?;
    Ok(Json(JobApplicationViewDto::new(&updated, offer.as_ref(), DEFAULT_STATUS)))
}

/// Remplace la lettre de motivation d'une candidature. Le corps est du texte brut (`text/plain`),
/// comme l'envoie le front.
#[utoipa::path(
    put,
    path = "/api/job-copilot/applications/{id}/cover-letter",
    tag = "job-copilot-applications",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Identifiant de la candidature")),
    request_body(content = String, description = "Lettre de motivation", content_type = "text/plain"),
    responses(
        (status = 200, description = "Lettre enregistree", body = JobApplicationViewDto),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Candidature inconnue")
    )
)]
pub async fn update_cover_letter(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    cover_letter: String,
) -> Result<Json<JobApplicationViewDto>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    find_owned(&mut conn, &auth.login, id)?;

    let now = Utc::now().naive_utc();
    let updated: JobApplication = diesel::update(job_application::table.find(id))
        .set((
            job_application::cover_letter.eq(cover_letter),
            job_application::updated_at.eq(now),
            job_application::last_modified_by.eq(auth.login.clone()),
            job_application::last_modified_date.eq(now),
        ))
        .returning(JobApplication::as_returning())
        .get_result(&mut conn)?;

    let offer = related_offer(&mut conn, &updated)?;
    Ok(Json(JobApplicationViewDto::new(&updated, offer.as_ref(), DEFAULT_STATUS)))
}

/// Genere une lettre pour une offre sans creer de candidature. Renvoie du texte brut.
#[utoipa::path(
    post,
    path = "/api/job-copilot/applications/{jobOfferId}/cover-letter",
    tag = "job-copilot-applications",
    security(("bearer_auth" = [])),
    params(
        ("jobOfferId" = Uuid, Path, description = "Offre visee"),
        ("language" = Option<String>, Query, description = "Langue de la lettre")
    ),
    responses(
        (status = 200, description = "Lettre generee", content_type = "text/plain"),
        (status = 400, description = "Offre inconnue ou profil candidat absent"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn generate_cover_letter(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(job_offer_id): Path<Uuid>,
    Query(params): Query<LanguageParam>,
) -> Result<String, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let (profile, offer) = {
        let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
        load_profile_and_offer(&mut conn, &auth.login, job_offer_id)?
    };

    CoverLetterService::generate(&profile, &offer, params.language.as_deref()).await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Normalise et valide un statut. Insensible a la casse, comme le `valueOf(toUpperCase())` Java.
fn normalize_status(raw: &str) -> Result<&'static str, AppError> {
    let candidate = raw.trim().to_uppercase();
    STATUSES
        .iter()
        .find(|status| **status == candidate)
        .copied()
        .ok_or_else(|| {
            AppError::BadRequest(format!(
                "Statut inconnu : {raw} (attendus : {})",
                STATUSES.join(", ")
            ))
        })
}

fn load_profile_and_offer(
    conn: &mut DbConnection,
    user_id: &str,
    job_offer_id: Uuid,
) -> Result<(CandidateProfile, JobOffer), AppError> {
    let profile: CandidateProfile = candidate_profile::table
        .filter(candidate_profile::user_id.eq(user_id))
        .order(candidate_profile::created_at.desc().nulls_last())
        .select(CandidateProfile::as_select())
        .first(conn)
        .optional()?
        .ok_or_else(|| {
            AppError::BadRequest("Aucun profil candidat : importer un CV d'abord".into())
        })?;

    let offer: JobOffer = job_offer::table
        .find(job_offer_id)
        .select(JobOffer::as_select())
        .first(conn)
        .optional()?
        .ok_or_else(|| AppError::BadRequest("Offre inconnue".into()))?;

    Ok((profile, offer))
}

/// Charge une candidature en verifiant l'appartenance : sans ce filtre, un identifiant devine
/// donnerait acces a la candidature de quelqu'un d'autre.
fn find_owned(
    conn: &mut DbConnection,
    user_id: &str,
    id: Uuid,
) -> Result<JobApplication, AppError> {
    job_application::table
        .find(id)
        .filter(job_application::user_id.eq(user_id))
        .select(JobApplication::as_select())
        .first(conn)
        .optional()?
        .ok_or_else(|| AppError::NotFound("Application not found".into()))
}

/// Recharge l'offre liee a une candidature, pour que la reponse porte la meme forme complete que
/// la liste (le front reutilise le meme composant pour afficher les deux).
fn related_offer(
    conn: &mut DbConnection,
    application: &JobApplication,
) -> Result<Option<JobOffer>, AppError> {
    let Some(offer_id) = application.jobOffer_id else {
        return Ok(None);
    };

    Ok(job_offer::table
        .find(offer_id)
        .select(JobOffer::as_select())
        .first(conn)
        .optional()?)
}

fn find_for_offer(
    conn: &mut DbConnection,
    user_id: &str,
    job_offer_id: Uuid,
) -> Result<Option<JobApplication>, AppError> {
    Ok(job_application::table
        .filter(job_application::user_id.eq(user_id))
        .filter(job_application::jobOffer_id.eq(job_offer_id))
        .select(JobApplication::as_select())
        .first(conn)
        .optional()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_status_is_case_insensitive() {
        assert_eq!(normalize_status("applied").unwrap(), "APPLIED");
        assert_eq!(normalize_status("  Interview  ").unwrap(), "INTERVIEW");
    }

    #[test]
    fn normalize_status_rejects_unknown_values() {
        // Un statut hors liste casserait l'union de chaines typee cote front.
        let error = normalize_status("PENDING").unwrap_err();
        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn statuses_cover_the_front_union_exactly() {
        // Garde-fou : le front type ApplicationStatus sur exactement ces sept valeurs.
        assert_eq!(STATUSES.len(), 7);
        assert!(STATUSES.contains(&DEFAULT_STATUS));
    }
}
