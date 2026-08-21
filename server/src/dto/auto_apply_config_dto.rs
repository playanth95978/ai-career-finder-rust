use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::AutoApplyConfig;

/// AutoApplyConfig DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AutoApplyConfigDto {
    pub id: i32,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_per_day: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<AutoApplyConfig> for AutoApplyConfigDto {
    fn from(entity: AutoApplyConfig) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            mode: entity.mode,
            min_score: entity.min_score,
            max_per_day: entity.max_per_day,
            sources: entity.sources,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new AutoApplyConfig
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAutoApplyConfigDto {
    #[validate(length(min = 1))]
    pub user_id: String,
    pub mode: Option<String>,
    pub min_score: Option<f64>,
    pub max_per_day: Option<i32>,
    pub sources: Option<String>,
}

impl CreateAutoApplyConfigDto {
}

/// DTO for updating a AutoApplyConfig
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAutoApplyConfigDto {
    pub user_id: Option<String>,
    pub mode: Option<String>,
    pub min_score: Option<f64>,
    pub max_per_day: Option<i32>,
    pub sources: Option<String>,
}

impl UpdateAutoApplyConfigDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod autoApplyConfig_dto_tests {
        use super::*;

        fn create_test_entity() -> AutoApplyConfig {
            AutoApplyConfig {
                id: 1,
                user_id: "test_value".to_string(),
                mode: Some("test_value".to_string()),
                min_score: Some(42.5),
                max_per_day: Some(42),
                sources: Some("test_value".to_string()),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_autoApplyConfig_dto_from_entity() {
            let entity = create_test_entity();
            let dto = AutoApplyConfigDto::from(entity);
            assert_eq!(dto.id, 1);
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_autoApplyConfig_dto_serialization() {
            let entity = create_test_entity();
            let dto = AutoApplyConfigDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":1"));
        }

        #[test]
        fn test_autoApplyConfig_dto_deserialization() {
            let json = r#"{"id":1,"userId":"test"}"#;
            let dto: AutoApplyConfigDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, 1);
        }
    }

    mod create_autoApplyConfig_dto_tests {
        use super::*;

        #[test]
        fn test_create_autoApplyConfig_dto_valid() {
            let dto = CreateAutoApplyConfigDto {
                user_id: "valid_value".to_string(),
                mode: None,
                min_score: None,
                max_per_day: None,
                sources: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_autoApplyConfig_dto_deserialization() {
            let json = r#"{"userId":"test_value"}"#;
            let result: Result<CreateAutoApplyConfigDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_autoApplyConfig_dto_empty_string_invalid() {
            let dto = CreateAutoApplyConfigDto {
                user_id: "".to_string(),
                mode: None,
                min_score: None,
                max_per_day: None,
                sources: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_autoApplyConfig_dto_tests {
        use super::*;

        #[test]
        fn test_update_autoApplyConfig_dto_all_none_valid() {
            let dto = UpdateAutoApplyConfigDto {
                user_id: None,
                mode: None,
                min_score: None,
                max_per_day: None,
                sources: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_autoApplyConfig_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateAutoApplyConfigDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
