use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;
use crate::dto::common::{deserialize_naive_datetime, deserialize_option_naive_datetime};

use crate::models::Conversation;
use uuid::Uuid;

/// Conversation DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConversationDto {
    pub id: Uuid,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_chat: Option<String>,
    #[serde(deserialize_with = "deserialize_naive_datetime")]
    pub created_at: NaiveDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub last_message_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<Conversation> for ConversationDto {
    fn from(entity: Conversation) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            title: entity.title,
            summary: entity.summary,
            metadata: entity.metadata,
            type_chat: entity.type_chat,
            created_at: entity.created_at,
            last_message_at: entity.last_message_at,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new Conversation
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationDto {
    #[validate(length(min = 1))]
    pub user_id: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub metadata: Option<String>,
    pub type_chat: Option<String>,
    #[serde(deserialize_with = "deserialize_naive_datetime")]
    pub created_at: NaiveDateTime,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub last_message_at: Option<NaiveDateTime>,
}

impl CreateConversationDto {
}

/// DTO for updating a Conversation
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConversationDto {
    pub user_id: Option<String>,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub metadata: Option<String>,
    pub type_chat: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub last_message_at: Option<NaiveDateTime>,
}

impl UpdateConversationDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod conversation_dto_tests {
        use super::*;

        fn create_test_entity() -> Conversation {
            Conversation {
                id: Uuid::nil(),
                user_id: "test_value".to_string(),
                title: Some("test_value".to_string()),
                summary: Some("test_value".to_string()),
                metadata: Some("test_value".to_string()),
                type_chat: Some("test_value".to_string()),
                created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
                last_message_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_conversation_dto_from_entity() {
            let entity = create_test_entity();
            let dto = ConversationDto::from(entity);
            assert_eq!(dto.id, Uuid::nil());
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_conversation_dto_serialization() {
            let entity = create_test_entity();
            let dto = ConversationDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
        }

        #[test]
        fn test_conversation_dto_deserialization() {
            let json = r#"{"id":"00000000-0000-0000-0000-000000000000","userId":"test","createdAt":"2024-01-01T00:00:00Z"}"#;
            let dto: ConversationDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, Uuid::nil());
        }
    }

    mod create_conversation_dto_tests {
        use super::*;

        #[test]
        fn test_create_conversation_dto_valid() {
            let dto = CreateConversationDto {
                user_id: "valid_value".to_string(),
                title: None,
                summary: None,
                metadata: None,
                type_chat: None,
                created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
                last_message_at: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_conversation_dto_deserialization() {
            let json = r#"{"userId":"test_value","createdAt":"2024-01-01T00:00:00Z"}"#;
            let result: Result<CreateConversationDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_conversation_dto_empty_string_invalid() {
            let dto = CreateConversationDto {
                user_id: "".to_string(),
                title: None,
                summary: None,
                metadata: None,
                type_chat: None,
                created_at: NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
                last_message_at: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_conversation_dto_tests {
        use super::*;

        #[test]
        fn test_update_conversation_dto_all_none_valid() {
            let dto = UpdateConversationDto {
                user_id: None,
                title: None,
                summary: None,
                metadata: None,
                type_chat: None,
                created_at: None,
                last_message_at: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_conversation_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateConversationDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
