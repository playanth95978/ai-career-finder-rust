use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;
use crate::dto::common::deserialize_option_naive_datetime;

use crate::models::CvResume;

/// CvResume DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CvResumeDto {
    pub id: i32,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    pub data: String,
    pub version_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<CvResume> for CvResumeDto {
    fn from(entity: CvResume) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            title: entity.title,
            template: entity.template,
            data: entity.data,
            version_number: entity.version_number,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new CvResume
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCvResumeDto {
    #[validate(length(min = 1))]
    pub user_id: String,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: String,
    pub version_number: i32,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}

impl CreateCvResumeDto {
}

/// DTO for updating a CvResume
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCvResumeDto {
    pub user_id: Option<String>,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: Option<String>,
    pub version_number: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}

impl UpdateCvResumeDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod cvResume_dto_tests {
        use super::*;

        fn create_test_entity() -> CvResume {
            CvResume {
                id: 1,
                user_id: "test_value".to_string(),
                title: Some("test_value".to_string()),
                template: Some("test_value".to_string()),
                data: "test_value".to_string(),
                version_number: 42,
                created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                updated_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_cvResume_dto_from_entity() {
            let entity = create_test_entity();
            let dto = CvResumeDto::from(entity);
            assert_eq!(dto.id, 1);
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_cvResume_dto_serialization() {
            let entity = create_test_entity();
            let dto = CvResumeDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":1"));
        }

        #[test]
        fn test_cvResume_dto_deserialization() {
            let json = r#"{"id":1,"userId":"test","data":"test","versionNumber":1}"#;
            let dto: CvResumeDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, 1);
        }
    }

    mod create_cvResume_dto_tests {
        use super::*;

        #[test]
        fn test_create_cvResume_dto_valid() {
            let dto = CreateCvResumeDto {
                user_id: "valid_value".to_string(),
                title: None,
                template: None,
                data: "valid_value".to_string(),
                version_number: 1,
                created_at: None,
                updated_at: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_cvResume_dto_deserialization() {
            let json = r#"{"userId":"test_value","data":"test_value","versionNumber":1}"#;
            let result: Result<CreateCvResumeDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_cvResume_dto_empty_string_invalid() {
            let dto = CreateCvResumeDto {
                user_id: "".to_string(),
                title: None,
                template: None,
                data: "valid".to_string(),
                version_number: 1,
                created_at: None,
                updated_at: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_cvResume_dto_tests {
        use super::*;

        #[test]
        fn test_update_cvResume_dto_all_none_valid() {
            let dto = UpdateCvResumeDto {
                user_id: None,
                title: None,
                template: None,
                data: None,
                version_number: None,
                created_at: None,
                updated_at: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_cvResume_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateCvResumeDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
