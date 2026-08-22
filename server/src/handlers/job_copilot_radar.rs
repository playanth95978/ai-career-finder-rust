//! Opportunity Radar (`/api/job-copilot/radar`) : le flux des offres proactivement matchees.
//!
//! Ecart assume par rapport a l'app Spring : seule la lecture du flux et les actions
//! (vu / ignore) sont portees. Le scan periodique qui *produit* les hits repose sur la recherche
//! vectorielle des offres, impossible tant que `job_offer` n'a pas de colonne `embedding`. Le flux
//! est donc alimente par ce que d'autres chemins ecrivent dans `radar_hit` ; les endpoints
//! repondent correctement, y compris sur un flux vide.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::{job_offer, radar_hit};
use crate::db::DbConnection;
use crate::dto::job_copilot_dto::{CountDto, JobOfferSummaryDto, MatchTagDto, RadarHitFeedDto};
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::{JobOffer, RadarHit, RoleType};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(feed))
        .route("/unread-count", get(unread_count))
        .route("/:id/seen", post(mark_seen))
        .route("/:id/dismiss", post(dismiss))
}

/// Flux radar de l'utilisateur : non vus d'abord, puis par score decroissant.
#[utoipa::path(
    get,
    path = "/api/job-copilot/radar",
    tag = "job-copilot-radar",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Flux radar", body = Vec<RadarHitFeedDto>),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn feed(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<RadarHitFeedDto>>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    // Jointure a gauche : un hit dont l'offre a ete purgee reste affichable plutot que de
    // disparaitre silencieusement du flux.
    let rows: Vec<(RadarHit, Option<JobOffer>)> = radar_hit::table
        .left_join(job_offer::table.on(job_offer::id.nullable().eq(radar_hit::jobOffer_id)))
        .filter(radar_hit::user_id.eq(&auth.login))
        .filter(radar_hit::dismissed.eq(false).or(radar_hit::dismissed.is_null()))
        .order((radar_hit::seen.asc().nulls_first(), radar_hit::score.desc().nulls_last()))
        .select((RadarHit::as_select(), Option::<JobOffer>::as_select()))
        .load(&mut conn)?;

    Ok(Json(rows.iter().map(|(hit, offer)| to_dto(hit, offer.as_ref())).collect()))
}

/// Nombre de hits non lus, pour le badge de navigation.
#[utoipa::path(
    get,
    path = "/api/job-copilot/radar/unread-count",
    tag = "job-copilot-radar",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Compteur de non-lus", body = CountDto),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn unread_count(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<CountDto>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let count: i64 = radar_hit::table
        .filter(radar_hit::user_id.eq(&auth.login))
        .filter(radar_hit::seen.eq(false).or(radar_hit::seen.is_null()))
        .filter(radar_hit::dismissed.eq(false).or(radar_hit::dismissed.is_null()))
        .count()
        .get_result(&mut conn)?;

    Ok(Json(CountDto { count }))
}

/// Marque un hit comme vu.
#[utoipa::path(
    post,
    path = "/api/job-copilot/radar/{id}/seen",
    tag = "job-copilot-radar",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Identifiant du hit")),
    responses(
        (status = 204, description = "Hit marque comme vu"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Hit inconnu pour cet utilisateur")
    )
)]
pub async fn mark_seen(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    set_flag(&state, &auth, id, Flag::Seen).await
}

/// Retire un hit du flux.
#[utoipa::path(
    post,
    path = "/api/job-copilot/radar/{id}/dismiss",
    tag = "job-copilot-radar",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Identifiant du hit")),
    responses(
        (status = 204, description = "Hit ignore"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Hit inconnu pour cet utilisateur")
    )
)]
pub async fn dismiss(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    set_flag(&state, &auth, id, Flag::Dismissed).await
}

enum Flag {
    Seen,
    Dismissed,
}

/// Le filtre sur `user_id` fait partie du `WHERE` de l'`UPDATE` : un identifiant appartenant a
/// quelqu'un d'autre ne doit pas etre modifiable, et se voit repondre 404 (et non 403, qui
/// confirmerait son existence).
async fn set_flag(
    state: &AppState,
    auth: &AuthUser,
    id: Uuid,
    flag: Flag,
) -> Result<StatusCode, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let target = radar_hit::table
        .find(id)
        .filter(radar_hit::user_id.eq(&auth.login));

    let updated = match flag {
        Flag::Seen => diesel::update(target)
            .set(radar_hit::seen.eq(true))
            .execute(&mut conn)?,
        Flag::Dismissed => diesel::update(target)
            .set(radar_hit::dismissed.eq(true))
            .execute(&mut conn)?,
    };

    if updated == 0 {
        return Err(AppError::NotFound("Radar hit not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn to_dto(hit: &RadarHit, offer: Option<&JobOffer>) -> RadarHitFeedDto {
    RadarHitFeedDto {
        id: hit.id,
        job_offer: offer.map(JobOfferSummaryDto::from),
        score: hit.score.unwrap_or(0.0),
        why_you: parse_why_you(hit.why_you.as_deref()),
        seen: hit.seen.unwrap_or(false),
        dismissed: hit.dismissed.unwrap_or(false),
        created_at: hit.created_at.map(|d| d.to_string()),
    }
}

/// `why_you` stocke la liste des tags de match en JSON. Un contenu illisible devient une liste
/// vide : un hit sans explication reste consultable, ce qui vaut mieux qu'un flux en erreur.
fn parse_why_you(raw: Option<&str>) -> Vec<MatchTagDto> {
    raw.map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|s| serde_json::from_str::<Vec<MatchTagDto>>(s).ok())
        .unwrap_or_default()
}

/// Aide de test : garde le type de connexion visible dans la signature des helpers ci-dessus.
#[allow(dead_code)]
fn _assert_connection_type(_: &mut DbConnection) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_why_you_reads_the_stored_tag_list() {
        let tags = parse_why_you(Some(r#"[{"key":"jobCopilot.match.tag.remote"}]"#));
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].key, "jobCopilot.match.tag.remote");
    }

    #[test]
    fn parse_why_you_falls_back_to_empty_on_unusable_json() {
        assert!(parse_why_you(None).is_empty());
        assert!(parse_why_you(Some("")).is_empty());
        assert!(parse_why_you(Some("[")).is_empty());
    }

    #[test]
    fn to_dto_defaults_nullable_flags_rather_than_omitting_them() {
        // Le front type `seen`/`dismissed`/`score` comme non-nuls : des colonnes NULL en base
        // doivent ressortir en false/0, pas disparaitre de la reponse.
        let hit = RadarHit {
            id: Uuid::nil(),
            user_id: "alice".to_string(),
            score: None,
            why_you: None,
            seen: None,
            dismissed: None,
            created_at: None,
            jobOffer_id: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let dto = to_dto(&hit, None);
        assert_eq!(dto.score, 0.0);
        assert!(!dto.seen);
        assert!(!dto.dismissed);
        assert!(dto.job_offer.is_none());
    }
}
