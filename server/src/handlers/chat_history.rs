//! Historique de conversation expose au front (`/api/chat-history`).
//!
//! Projection plate de la table `chat_message`, dont l'agent lit par ailleurs la forme complete
//! via [`crate::services::conversation_memory::PostgresConversationMemory`]. Un seul stockage,
//! deux lectures.

use axum::{
    extract::{Path, State},
    routing::get,
    Extension, Json, Router,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::chat_message::ChatMessage;
use crate::models::RoleType;
use crate::services::conversation_memory::PostgresConversationMemory;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/history", get(get_history))
        .route("/history/:id", get(get_conversation_history))
}

/// Un message d'historique, forme attendue par `ChatMessageHistory` cote Angular.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChatHistoryDto {
    pub conversation_id: String,
    pub content: String,
    /// `USER`, `ASSISTANT` ou `SYSTEM`.
    pub r#type: String,
    pub timestamp: String,
}

impl From<&ChatMessage> for ChatHistoryDto {
    fn from(row: &ChatMessage) -> Self {
        Self {
            conversation_id: row.conversation_id.clone(),
            content: row.content.clone(),
            r#type: row.role.clone(),
            timestamp: row.created_at.and_utc().to_rfc3339(),
        }
    }
}

/// Historique complet de l'utilisateur, toutes conversations confondues.
#[utoipa::path(
    get,
    path = "/api/chat-history/history",
    tag = "chat-history",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Messages de l'utilisateur", body = Vec<ChatHistoryDto>),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_history(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<ChatHistoryDto>>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = PostgresConversationMemory::all_rows_for_user(&mut conn, &auth.login)?;

    Ok(Json(to_dtos(&rows)))
}

/// Historique d'une conversation precise.
///
/// Le filtre porte aussi sur l'utilisateur : sans lui, un identifiant de conversation devine
/// donnerait acces a la conversation de quelqu'un d'autre. Une conversation inconnue *ou*
/// appartenant a un autre utilisateur renvoie 404 — un 403 confirmerait son existence.
#[utoipa::path(
    get,
    path = "/api/chat-history/history/{id}",
    tag = "chat-history",
    security(("bearer_auth" = [])),
    params(("id" = String, Path, description = "Identifiant de la conversation")),
    responses(
        (status = 200, description = "Messages de la conversation", body = Vec<ChatHistoryDto>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Conversation inconnue pour cet utilisateur")
    )
)]
pub async fn get_conversation_history(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ChatHistoryDto>>, AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let mut conn = state.pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = PostgresConversationMemory::rows(&mut conn, &auth.login, &id)?;

    if rows.is_empty() {
        return Err(AppError::NotFound("Conversation not found".into()));
    }

    Ok(Json(to_dtos(&rows)))
}

/// Convertit les lignes en DTO en ecartant les messages sans texte.
///
/// Un appel d'outil et son resultat sont des messages a part entiere pour l'agent, mais n'ont
/// aucun texte a afficher : les laisser passer ferait apparaitre des bulles vides dans le fil.
fn to_dtos(rows: &[ChatMessage]) -> Vec<ChatHistoryDto> {
    rows.iter()
        .filter(|row| !row.content.trim().is_empty())
        .map(ChatHistoryDto::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn row(role: &str, content: &str, sequence: i32) -> ChatMessage {
        ChatMessage {
            id: Uuid::nil(),
            conversation_id: "conv-1".to_string(),
            user_id: "alice".to_string(),
            sequence,
            role: role.to_string(),
            content: content.to_string(),
            payload: "{}".to_string(),
            created_at: NaiveDate::from_ymd_opt(2026, 8, 22)
                .unwrap()
                .and_hms_opt(10, 30, 0)
                .unwrap(),
        }
    }

    #[test]
    fn to_dtos_drops_messages_without_displayable_text() {
        // Le message intermediaire est un appel d'outil : reel pour l'agent, vide a l'ecran.
        let rows = vec![
            row("USER", "trouve-moi un poste", 0),
            row("ASSISTANT", "", 1),
            row("USER", "   ", 2),
            row("ASSISTANT", "voici trois offres", 3),
        ];
        let dtos = to_dtos(&rows);
        assert_eq!(dtos.len(), 2);
        assert_eq!(dtos[0].content, "trouve-moi un poste");
        assert_eq!(dtos[1].content, "voici trois offres");
    }

    #[test]
    fn dto_uses_the_field_names_the_front_expects() {
        let json = serde_json::to_value(ChatHistoryDto::from(&row("USER", "salut", 0))).unwrap();
        let object = json.as_object().unwrap();
        // `type` et non `role` : c'est le nom que porte ChatMessageHistory cote Angular.
        for field in ["conversationId", "content", "type", "timestamp"] {
            assert!(object.contains_key(field), "{field} attendu");
        }
        assert_eq!(object["type"], "USER");
    }

    #[test]
    fn timestamp_is_serialised_as_rfc3339() {
        // Le front fait `new Date(timestamp)` : un horodatage sans fuseau serait interprete en
        // heure locale du navigateur, decalant tout le fil.
        let dto = ChatHistoryDto::from(&row("USER", "salut", 0));
        assert_eq!(dto.timestamp, "2026-08-22T10:30:00+00:00");
    }

    #[test]
    fn to_dtos_preserves_the_stored_order() {
        let rows = vec![
            row("USER", "premier", 0),
            row("ASSISTANT", "deuxieme", 1),
            row("USER", "troisieme", 2),
        ];
        let dtos = to_dtos(&rows);
        let contents: Vec<&str> = dtos.iter().map(|d| d.content.as_str()).collect();
        assert_eq!(contents, vec!["premier", "deuxieme", "troisieme"]);
    }
}
