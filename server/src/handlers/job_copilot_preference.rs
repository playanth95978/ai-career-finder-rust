//! Preferences de recherche du Job Copilot (`/api/job-copilot/preferences`).
//!
//! Le front envoie les listes en tableaux et les relit sur l'entite : `GET` renvoie donc
//! l'entite `UserPreference` (colonnes TEXT contenant du JSON), exactement comme la version Java.

use axum::{
    extract::State,
    routing::get,
    Extension, Json, Router,
};
use chrono::Utc;
use diesel::prelude::*;

use crate::db::schema::user_preference;
use crate::dto::job_copilot_dto::{to_json_text, UserPreferenceInputDto};
use crate::dto::UserPreferenceDto;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::{NewUserPreference, RoleType, UserPreference};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(get_preferences).put(update_preferences))
}

/// Preferences de l'utilisateur courant, 404 s'il n'en a jamais enregistre.
#[utoipa::path(
    get,
    path = "/api/job-copilot/preferences",
    tag = "job-copilot-preferences",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Preferences courantes", body = UserPreferenceDto),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Aucune preference enregistree")
    )
)]
pub async fn get_preferences(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<UserPreferenceDto>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let found = find_for_user(&mut conn, &auth.login)?;

    found
        .map(|p| Json(UserPreferenceDto::from(p)))
        .ok_or_else(|| AppError::NotFound("No preferences for this user".into()))
}

/// Enregistre les preferences (upsert). Le corps porte des tableaux, la base des colonnes TEXT.
#[utoipa::path(
    put,
    path = "/api/job-copilot/preferences",
    tag = "job-copilot-preferences",
    security(("bearer_auth" = [])),
    request_body = UserPreferenceInputDto,
    responses(
        (status = 200, description = "Preferences enregistrees", body = UserPreferenceDto),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn update_preferences(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(dto): Json<UserPreferenceInputDto>,
) -> Result<Json<UserPreferenceDto>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let now = Utc::now().naive_utc();

    let preferred_roles = to_json_text(&dto.preferred_roles);
    let excluded_technologies = to_json_text(&dto.excluded_technologies);
    let preferred_locations = to_json_text(&dto.preferred_locations);

    let saved: UserPreference = match find_for_user(&mut conn, &auth.login)? {
        Some(existing) => diesel::update(user_preference::table.find(existing.id))
            .set((
                user_preference::remote_only.eq(dto.remote_only),
                user_preference::contract_type.eq(dto.contract_type),
                user_preference::salary_min.eq(dto.salary_min),
                user_preference::salary_max.eq(dto.salary_max),
                user_preference::preferred_roles.eq(preferred_roles),
                user_preference::excluded_technologies.eq(excluded_technologies),
                user_preference::preferred_locations.eq(preferred_locations),
                user_preference::last_modified_by.eq(auth.login.clone()),
                user_preference::last_modified_date.eq(now),
            ))
            .returning(UserPreference::as_returning())
            .get_result(&mut conn)?,
        None => {
            let new_preference = NewUserPreference {
                user_id: auth.login.clone(),
                remote_only: dto.remote_only,
                contract_type: dto.contract_type,
                salary_min: dto.salary_min,
                salary_max: dto.salary_max,
                preferred_roles,
                excluded_technologies,
                preferred_locations,
                created_by: Some(auth.login.clone()),
                created_date: Some(now),
                last_modified_by: Some(auth.login.clone()),
                last_modified_date: Some(now),
            };
            diesel::insert_into(user_preference::table)
                .values(&new_preference)
                .returning(UserPreference::as_returning())
                .get_result(&mut conn)?
        }
    };

    Ok(Json(UserPreferenceDto::from(saved)))
}

fn find_for_user(
    conn: &mut crate::db::DbConnection,
    user_id: &str,
) -> Result<Option<UserPreference>, AppError> {
    Ok(user_preference::table
        .filter(user_preference::user_id.eq(user_id))
        .select(UserPreference::as_select())
        .first(conn)
        .optional()?)
}
