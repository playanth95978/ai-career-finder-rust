use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::UserPreference;
use uuid::Uuid;

/// UserPreference DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferenceDto {
    pub id: Uuid,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salary_min: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salary_max: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_roles: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_technologies: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_locations: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<UserPreference> for UserPreferenceDto {
    fn from(entity: UserPreference) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            remote_only: entity.remote_only,
            contract_type: entity.contract_type,
            salary_min: entity.salary_min,
            salary_max: entity.salary_max,
            preferred_roles: entity.preferred_roles,
            excluded_technologies: entity.excluded_technologies,
            preferred_locations: entity.preferred_locations,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new UserPreference
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserPreferenceDto {
    #[validate(length(min = 1))]
    pub user_id: String,
    pub remote_only: Option<bool>,
    pub contract_type: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub preferred_roles: Option<String>,
    pub excluded_technologies: Option<String>,
    pub preferred_locations: Option<String>,
}

impl CreateUserPreferenceDto {
}

/// DTO for updating a UserPreference
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserPreferenceDto {
    pub user_id: Option<String>,
    pub remote_only: Option<bool>,
    pub contract_type: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub preferred_roles: Option<String>,
    pub excluded_technologies: Option<String>,
    pub preferred_locations: Option<String>,
}

impl UpdateUserPreferenceDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod userPreference_dto_tests {
        use super::*;

        fn create_test_entity() -> UserPreference {
            UserPreference {
                id: Uuid::nil(),
                user_id: "test_value".to_string(),
                remote_only: Some(true),
                contract_type: Some("test_value".to_string()),
                salary_min: Some(42),
                salary_max: Some(42),
                preferred_roles: Some("test_value".to_string()),
                excluded_technologies: Some("test_value".to_string()),
                preferred_locations: Some("test_value".to_string()),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_userPreference_dto_from_entity() {
            let entity = create_test_entity();
            let dto = UserPreferenceDto::from(entity);
            assert_eq!(dto.id, Uuid::nil());
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_userPreference_dto_serialization() {
            let entity = create_test_entity();
            let dto = UserPreferenceDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
        }

        #[test]
        fn test_userPreference_dto_deserialization() {
            let json = r#"{"id":"00000000-0000-0000-0000-000000000000","userId":"test"}"#;
            let dto: UserPreferenceDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, Uuid::nil());
        }
    }

    mod create_userPreference_dto_tests {
        use super::*;

        #[test]
        fn test_create_userPreference_dto_valid() {
            let dto = CreateUserPreferenceDto {
                user_id: "valid_value".to_string(),
                remote_only: None,
                contract_type: None,
                salary_min: None,
                salary_max: None,
                preferred_roles: None,
                excluded_technologies: None,
                preferred_locations: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_userPreference_dto_deserialization() {
            let json = r#"{"userId":"test_value"}"#;
            let result: Result<CreateUserPreferenceDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_userPreference_dto_empty_string_invalid() {
            let dto = CreateUserPreferenceDto {
                user_id: "".to_string(),
                remote_only: None,
                contract_type: None,
                salary_min: None,
                salary_max: None,
                preferred_roles: None,
                excluded_technologies: None,
                preferred_locations: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_userPreference_dto_tests {
        use super::*;

        #[test]
        fn test_update_userPreference_dto_all_none_valid() {
            let dto = UpdateUserPreferenceDto {
                user_id: None,
                remote_only: None,
                contract_type: None,
                salary_min: None,
                salary_max: None,
                preferred_roles: None,
                excluded_technologies: None,
                preferred_locations: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_userPreference_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateUserPreferenceDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
