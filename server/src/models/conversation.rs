use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Conversation entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::conversation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Conversation {
    pub id: Uuid,
    pub user_id: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub metadata: Option<String>,
    pub type_chat: Option<String>,
    pub created_at: NaiveDateTime,
    pub last_message_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New Conversation for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::conversation)]
pub struct NewConversation {
    pub user_id: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub metadata: Option<String>,
    pub type_chat: Option<String>,
    pub created_at: NaiveDateTime,
    pub last_message_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// Conversation update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::conversation)]
pub struct UpdateConversation {
    pub user_id: Option<String>,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub metadata: Option<String>,
    pub type_chat: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub last_message_at: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_clone() {
        let entity = Conversation {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            title: None,
            summary: None,
            metadata: None,
            type_chat: None,
            created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            last_message_at: None,
            created_by: Some("system".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let cloned = entity.clone();
        assert_eq!(entity.id, cloned.id);
    }

    #[test]
    fn test_conversation_debug() {
        let entity = Conversation {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            title: None,
            summary: None,
            metadata: None,
            type_chat: None,
            created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            last_message_at: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("Conversation"));
    }

    #[test]
    fn test_new_conversation_creation() {
        let new_entity = NewConversation {
            user_id: "test".to_string(),
            title: None,
            summary: None,
            metadata: None,
            type_chat: None,
            created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            last_message_at: None,
            created_by: Some("test_user".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        assert_eq!(new_entity.created_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_update_conversation_creation() {
        let update = UpdateConversation {
            user_id: Some("updated".to_string()),
            title: Some("updated".to_string()),
            summary: Some("updated".to_string()),
            metadata: Some("updated".to_string()),
            type_chat: Some("updated".to_string()),
            created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            last_message_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_conversation_serialization() {
        let entity = Conversation {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            title: None,
            summary: None,
            metadata: None,
            type_chat: None,
            created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            last_message_at: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
    }
}
