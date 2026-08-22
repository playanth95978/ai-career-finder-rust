use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::chat_message;

/// Un message d'historique de conversation.
///
/// `payload` porte le message rig serialise en entier (appels d'outils et resultats inclus) ;
/// `role` et `content` en sont la projection texte, denormalisee pour que l'endpoint d'historique
/// n'ait pas a parser du JSON ligne par ligne.
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = chat_message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatMessage {
    pub id: Uuid,
    pub conversation_id: String,
    pub user_id: String,
    pub sequence: i32,
    pub role: String,
    pub content: String,
    pub payload: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = chat_message)]
pub struct NewChatMessage {
    pub id: Uuid,
    pub conversation_id: String,
    pub user_id: String,
    pub sequence: i32,
    pub role: String,
    pub content: String,
    pub payload: String,
    pub created_at: NaiveDateTime,
}

/// Roles persistes. Miroir des variantes de `rig::completion::Message`, en majuscules parce que
/// c'est la forme que l'union de chaines du front attend (`'USER' | 'ASSISTANT' | 'SYSTEM'`).
pub mod chat_role {
    pub const USER: &str = "USER";
    pub const ASSISTANT: &str = "ASSISTANT";
    pub const SYSTEM: &str = "SYSTEM";
}
