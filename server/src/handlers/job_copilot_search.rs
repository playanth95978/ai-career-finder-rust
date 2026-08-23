//! Recherche et matching d'offres (`/api/job-copilot/jobs`).
//!
//! `search-smart`, `match` et `match-advanced` classent le corpus par similarite vectorielle
//! (pgvector, via [`crate::services::job_offer_vector_index::JobOfferVectorIndex`]), avec repli
//! lexical tant qu'une offre n'est pas vectorisee.
//!
//! Ecart restant avec l'app Spring : celle-ci fusionne le vectoriel et BM25 par Reciprocal Rank
//! Fusion puis reclasse avec un cross-encoder. Le RRF et le reranking ne sont pas portes — rig
//! n'expose de client de reranking que pour VoyageAI, pas pour le provider utilise ici.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::schema::{candidate_profile, job_offer, user_preference};
use crate::db::DbConnection;
use crate::dto::job_copilot_dto::{parse_string_list, CountDto, MatchResultDto};
use crate::dto::JobOfferDto;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::{CandidateProfile, JobOffer, RoleType, UserPreference};
use crate::services::job_matching_service::JobMatchingService;
use crate::services::job_search_service::JobSearchService;
use crate::AppState;

/// Nombre d'offres classees par defaut, aligne sur les valeurs par defaut du front.
const DEFAULT_LIMIT: i64 = 30;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/search", get(search))
        .route("/search-smart", get(search_smart))
        .route("/indexed-count", get(indexed_count))
        .route("/match", get(match_jobs))
        .route("/match-advanced", get(match_advanced))
        .route("/_search", get(search_by_criteria))
        .route("/fetch", post(fetch_from_url))
        .route("/:id/enrich", post(enrich))
}

// ---------------------------------------------------------------------------
// Parametres de requete
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub keywords: Option<String>,
    pub location: Option<String>,
    pub source: Option<String>,
    /// Marche demande (fr, gb, us, au…). Selectionne le marche des connecteurs partitionnes par
    /// pays (Adzuna, Careerjet) et sert de preuve pour ecarter les offres d'un autre pays lors du
    /// reclassement geographique.
    pub country: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartSearchParams {
    pub query: String,
    pub location: Option<String>,
    pub source: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchParams {
    pub top_k: Option<i64>,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CriteriaParams {
    /// Critere JHipster `applyUrl.equals`, le seul que le front emette sur cet endpoint : il
    /// resout une offre (et son identifiant) depuis un apercu qui n'a que son URL.
    #[serde(rename = "applyUrl.equals")]
    pub apply_url_equals: Option<String>,
    pub size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UrlParam {
    pub url: String,
}

// ---------------------------------------------------------------------------
// Endpoints
// ---------------------------------------------------------------------------

/// Recherche live chez les sources externes. Les offres nouvelles sont persistees au passage, ce
/// qui leur donne l'identifiant dont le front a besoin pour candidater.
#[utoipa::path(
    get,
    path = "/api/job-copilot/jobs/search",
    tag = "job-copilot-jobs",
    security(("bearer_auth" = [])),
    params(
        ("keywords" = Option<String>, Query, description = "Mots-cles"),
        ("location" = Option<String>, Query, description = "Ville"),
        ("source" = Option<String>, Query, description = "Restreint a une source"),
        ("country" = Option<String>, Query, description = "Marche (fr, gb, us, au…) : selectionne le marche des sources partitionnees et filtre les autres pays")
    ),
    responses(
        (status = 200, description = "Offres trouvees", body = Vec<JobOfferDto>),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn search(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<JobOfferDto>>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let offers = JobSearchService::search_all(
        &mut conn,
        params.keywords.as_deref(),
        params.location.as_deref(),
        params.source.as_deref(),
        params.country.as_deref(),
    )
    .await?;

    Ok(Json(offers.into_iter().map(JobOfferDto::from).collect()))
}

/// Recherche semantique sur le corpus deja ingere, sans appel aux sources externes.
#[utoipa::path(
    get,
    path = "/api/job-copilot/jobs/search-smart",
    tag = "job-copilot-jobs",
    security(("bearer_auth" = [])),
    params(
        ("query" = String, Query, description = "Requete en langage naturel"),
        ("location" = Option<String>, Query, description = "Lieu, ajoute a la requete"),
        ("source" = Option<String>, Query, description = "Restreint a une source"),
        ("limit" = Option<i64>, Query, description = "Nombre maximum de resultats")
    ),
    responses(
        (status = 200, description = "Offres classees", body = Vec<JobOfferDto>),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn search_smart(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(params): Query<SmartSearchParams>,
) -> Result<Json<Vec<JobOfferDto>>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    // Le lieu est concatene a la requete plutot que filtre durement : il biaise le classement
    // sans exclure une offre voisine qui reste pertinente. Comportement identique au Java.
    let query = match params.location.as_deref().map(str::trim).filter(|l| !l.is_empty()) {
        Some(location) => format!("{} {}", params.query, location),
        None => params.query.clone(),
    };

    let offers = JobSearchService::search_semantic(
        &state.job_offer_index,
        &state.pool,
        &query,
        params.source.as_deref(),
        params.limit.unwrap_or(DEFAULT_LIMIT),
    )
    .await?;

    Ok(Json(offers.into_iter().map(JobOfferDto::from).collect()))
}

/// Nombre d'offres reellement indexees, utilise comme statistique « taille du corpus ».
#[utoipa::path(
    get,
    path = "/api/job-copilot/jobs/indexed-count",
    tag = "job-copilot-jobs",
    responses((status = 200, description = "Nombre d'offres indexees", body = CountDto))
)]
pub async fn indexed_count(
    State(state): State<AppState>,
) -> Result<Json<CountDto>, AppError> {
    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(CountDto {
        count: JobSearchService::indexed_count(&mut conn)?,
    }))
}

/// Offres classees puis scorees contre le profil de l'utilisateur.
#[utoipa::path(
    get,
    path = "/api/job-copilot/jobs/match",
    tag = "job-copilot-jobs",
    security(("bearer_auth" = [])),
    params(
        ("topK" = Option<i64>, Query, description = "Nombre d'offres pre-selectionnees"),
        ("location" = Option<String>, Query, description = "Lieu recherche, prioritaire sur le CV")
    ),
    responses(
        (status = 200, description = "Offres scorees", body = Vec<MatchResultDto>),
        (status = 400, description = "Aucun profil : importer un CV d'abord"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn match_jobs(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(params): Query<MatchParams>,
) -> Result<Json<Vec<MatchResultDto>>, AppError> {
    run_match(&state, &auth, params.top_k.unwrap_or(DEFAULT_LIMIT), params.location.as_deref()).await
}

/// Variante « avancee ». Elle partage le pipeline de `match` : la version Java les differencie par
/// la strategie de recuperation (cosinus seul contre hybride reranke), et c'est le reranking qui
/// manque ici — pas les embeddings.
#[utoipa::path(
    get,
    path = "/api/job-copilot/jobs/match-advanced",
    tag = "job-copilot-jobs",
    security(("bearer_auth" = [])),
    params(("location" = Option<String>, Query, description = "Lieu recherche")),
    responses(
        (status = 200, description = "Offres scorees", body = Vec<MatchResultDto>),
        (status = 400, description = "Aucun profil : importer un CV d'abord"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn match_advanced(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(params): Query<MatchParams>,
) -> Result<Json<Vec<MatchResultDto>>, AppError> {
    run_match(&state, &auth, DEFAULT_LIMIT, params.location.as_deref()).await
}

async fn run_match(
    state: &AppState,
    auth: &AuthUser,
    top_k: i64,
    location: Option<&str>,
) -> Result<Json<Vec<MatchResultDto>>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

    let profile = latest_profile(&mut conn, &auth.login)?.ok_or_else(|| {
        AppError::BadRequest("Aucun profil candidat : importer un CV d'abord".into())
    })?;
    let preferences = preferences_for(&mut conn, &auth.login)?;

    // Pre-filtrage par pertinence plutot que scoring de tout le corpus : sinon on renvoie des
    // offres sans rapport, avec un score faible mais bien presentes dans la liste.
    //
    // Le pre-filtrage est semantique : c'est ce qui permet a un profil « DevOps » de remonter une
    // annonce « ingenieur infrastructure », que le lexical manquait faute de mot commun.
    let query = profile_query(&profile);
    let offers = JobSearchService::search_semantic(
        &state.job_offer_index,
        &state.pool,
        &query,
        None,
        top_k.max(1),
    )
    .await?;

    let results = JobMatchingService::match_jobs(&profile, &offers, preferences.as_ref(), location);
    Ok(Json(results))
}

/// Resout une offre depuis ses criteres. Renvoie `X-Total-Count`, comme la pagination JHipster.
#[utoipa::path(
    get,
    path = "/api/job-copilot/jobs/_search",
    tag = "job-copilot-jobs",
    security(("bearer_auth" = [])),
    params(
        ("applyUrl.equals" = Option<String>, Query, description = "URL de candidature exacte"),
        ("size" = Option<i64>, Query, description = "Taille de page")
    ),
    responses(
        (status = 200, description = "Offres correspondantes", body = Vec<JobOfferDto>),
        (status = 400, description = "Aucun critere supporte fourni"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn search_by_criteria(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(params): Query<CriteriaParams>,
) -> Result<impl IntoResponse, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    // Un critere non supporte renverrait sinon le corpus entier, ce que le front interpreterait
    // comme « cette URL correspond a n'importe quelle offre ».
    let apply_url = params
        .apply_url_equals
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            AppError::BadRequest(
                "Critere requis : applyUrl.equals (seul critere supporte sur cet endpoint)".into(),
            )
        })?;

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let (offers, total) =
        JobSearchService::find_by_apply_url(&mut conn, apply_url, params.size.unwrap_or(20))?;

    let mut headers = HeaderMap::new();
    if let Ok(value) = HeaderValue::from_str(&total.to_string()) {
        headers.insert("X-Total-Count", value);
    }

    let dtos: Vec<JobOfferDto> = offers.into_iter().map(JobOfferDto::from).collect();
    Ok((headers, Json(dtos)))
}

/// Importe les offres publiees sur une URL de board supportee.
#[utoipa::path(
    post,
    path = "/api/job-copilot/jobs/fetch",
    tag = "job-copilot-jobs",
    security(("bearer_auth" = [])),
    params(("url" = String, Query, description = "URL du board ou du flux")),
    responses(
        (status = 200, description = "Offres importees", body = Vec<JobOfferDto>),
        (status = 400, description = "URL vide ou non geree"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn fetch_from_url(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(params): Query<UrlParam>,
) -> Result<Json<Vec<JobOfferDto>>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let offers = JobSearchService::fetch_from_url(&mut conn, &params.url).await?;

    Ok(Json(offers.into_iter().map(JobOfferDto::from).collect()))
}

/// Complete une offre partielle depuis sa page source.
///
/// Idempotent, et pour l'instant sans effet : le seul connecteur porte (emploi.nc) fournit deja la
/// description complete des le listing, il n'y a donc rien a aller rechercher. L'endpoint existe
/// parce que le front l'appelle a l'ouverture d'une offre ; il renvoie l'offre telle quelle.
#[utoipa::path(
    post,
    path = "/api/job-copilot/jobs/{id}/enrich",
    tag = "job-copilot-jobs",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Identifiant de l'offre")),
    responses(
        (status = 200, description = "Offre, enrichie si la source le permettait", body = JobOfferDto),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Offre inconnue")
    )
)]
pub async fn enrich(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<JobOfferDto>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let offer: Option<JobOffer> = job_offer::table
        .find(id)
        .select(JobOffer::as_select())
        .first(&mut conn)
        .optional()?;

    offer
        .map(|o| Json(JobOfferDto::from(o)))
        .ok_or_else(|| AppError::NotFound("Job offer not found".into()))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn latest_profile(
    conn: &mut DbConnection,
    user_id: &str,
) -> Result<Option<CandidateProfile>, AppError> {
    Ok(candidate_profile::table
        .filter(candidate_profile::user_id.eq(user_id))
        .order(candidate_profile::created_at.desc().nulls_last())
        .select(CandidateProfile::as_select())
        .first(conn)
        .optional()?)
}

fn preferences_for(
    conn: &mut DbConnection,
    user_id: &str,
) -> Result<Option<UserPreference>, AppError> {
    Ok(user_preference::table
        .filter(user_preference::user_id.eq(user_id))
        .select(UserPreference::as_select())
        .first(conn)
        .optional()?)
}

/// Construit la requete de pre-filtrage a partir du profil.
///
/// Seuls les postes recherches et les competences sont utilises : le markdown integral du CV
/// noierait le classement lexical sous des mots communs (adresse, formules, noms d'ecoles) et
/// ramenerait des offres sans rapport.
fn profile_query(profile: &CandidateProfile) -> String {
    let mut terms = parse_string_list(profile.preferred_roles.as_deref());
    terms.extend(parse_string_list(profile.skills.as_deref()));
    terms.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(roles: Option<&str>, skills: Option<&str>) -> CandidateProfile {
        CandidateProfile {
            id: Uuid::nil(),
            user_id: "alice".to_string(),
            full_name: None,
            email: None,
            location: None,
            years_of_experience: None,
            skills: skills.map(str::to_owned),
            experiences: None,
            preferred_roles: roles.map(str::to_owned),
            languages: None,
            education: None,
            certifications: None,
            raw_markdown: Some("Adresse, formules de politesse et autres mots communs".to_string()),
            cv_filename: None,
            embedding_model: None,
            embedded_at: None,
            created_at: None,
            updated_at: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        }
    }

    #[test]
    fn profile_query_joins_roles_then_skills() {
        let query = profile_query(&profile(Some(r#"["Backend"]"#), Some(r#"["rust","axum"]"#)));
        assert_eq!(query, "Backend rust axum");
    }

    #[test]
    fn profile_query_excludes_the_raw_cv_markdown() {
        // Le markdown est volontairement laisse de cote : il diluerait le classement lexical.
        let query = profile_query(&profile(Some(r#"["Backend"]"#), None));
        assert!(!query.contains("politesse"));
    }

    #[test]
    fn profile_query_is_empty_when_the_profile_carries_no_usable_terms() {
        // Un profil sans role ni competence produit une requete vide, que la recherche traite
        // comme « offres les plus recentes » plutot que comme une erreur.
        assert!(profile_query(&profile(None, None)).is_empty());
    }
}
