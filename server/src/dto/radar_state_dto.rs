use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;
use crate::dto::common::deserialize_option_naive_datetime;

use crate::models::RadarState;
use uuid::Uuid;

/// RadarState DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RadarStateDto {
    pub id: Uuid,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub last_offer_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<RadarState> for RadarStateDto {
    fn from(entity: RadarState) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            last_offer_at: entity.last_offer_at,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new RadarState
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRadarStateDto {
    #[validate(length(min = 1))]
    pub user_id: String,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub last_offer_at: Option<NaiveDateTime>,
}

impl CreateRadarStateDto {
}

/// DTO for updating a RadarState
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRadarStateDto {
    pub user_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub last_offer_at: Option<NaiveDateTime>,
}

impl UpdateRadarStateDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod radarState_dto_tests {
        use super::*;

        fn create_test_entity() -> RadarState {
            RadarState {
                id: Uuid::nil(),
                user_id: "test_value".to_string(),
                last_offer_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_radarState_dto_from_entity() {
            let entity = create_test_entity();
            let dto = RadarStateDto::from(entity);
            assert_eq!(dto.id, Uuid::nil());
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_radarState_dto_serialization() {
            let entity = create_test_entity();
            let dto = RadarStateDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
        }

        #[test]
        fn test_radarState_dto_deserialization() {
            let json = r#"{"id":"00000000-0000-0000-0000-000000000000","userId":"test"}"#;
            let dto: RadarStateDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, Uuid::nil());
        }
    }

    mod create_radarState_dto_tests {
        use super::*;

        #[test]
        fn test_create_radarState_dto_valid() {
            let dto = CreateRadarStateDto {
                user_id: "valid_value".to_string(),
                last_offer_at: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_radarState_dto_deserialization() {
            let json = r#"{"userId":"test_value"}"#;
            let result: Result<CreateRadarStateDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_radarState_dto_empty_string_invalid() {
            let dto = CreateRadarStateDto {
                user_id: "".to_string(),
                last_offer_at: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_radarState_dto_tests {
        use super::*;

        #[test]
        fn test_update_radarState_dto_all_none_valid() {
            let dto = UpdateRadarStateDto {
                user_id: None,
                last_offer_at: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_radarState_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateRadarStateDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
