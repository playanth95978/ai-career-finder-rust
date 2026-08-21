use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;
use crate::dto::common::deserialize_option_naive_datetime;
use crate::dto::common::deserialize_optional_relationship;

use crate::models::{CvResumeVersion, CvResume};
use uuid::Uuid;

/// Minimal DTO for CvResume relationship in CvResumeVersion responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CvResumeVersionCvResumeRelationDto {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl From<CvResume> for CvResumeVersionCvResumeRelationDto {
    fn from(entity: CvResume) -> Self {
        Self {
            id: entity.id,
            title: entity.title,
        }
    }
}

/// CvResumeVersion DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CvResumeVersionDto {
    pub id: Uuid,
    pub version_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<CvResumeVersionCvResumeRelationDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<CvResumeVersion> for CvResumeVersionDto {
    fn from(entity: CvResumeVersion) -> Self {
        Self {
            id: entity.id,
            version_number: entity.version_number,
            title: entity.title,
            template: entity.template,
            data: entity.data,
            created_at: entity.created_at,
            resume: None,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

impl CvResumeVersionDto {
    /// Create a DTO from entity with related entities
    pub fn from_with_relations(
        entity: CvResumeVersion,
        resume: Option<CvResume>,
    ) -> Self {
        Self {
            id: entity.id,
            version_number: entity.version_number,
            title: entity.title,
            template: entity.template,
            data: entity.data,
            created_at: entity.created_at,
            resume: resume.map(CvResumeVersionCvResumeRelationDto::from),
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new CvResumeVersion
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCvResumeVersionDto {
    pub version_number: i32,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: String,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, rename = "resume", deserialize_with = "deserialize_optional_relationship")]
    pub resume_id: Option<Uuid>,
}

impl CreateCvResumeVersionDto {
}

/// DTO for updating a CvResumeVersion
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCvResumeVersionDto {
    pub version_number: Option<i32>,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, rename = "resume", deserialize_with = "deserialize_optional_relationship")]
    pub resume_id: Option<Uuid>,
}

impl UpdateCvResumeVersionDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod cvResumeVersion_dto_tests {
        use super::*;

        fn create_test_entity() -> CvResumeVersion {
            CvResumeVersion {
                id: Uuid::nil(),
                version_number: 42,
                title: Some("test_value".to_string()),
                template: Some("test_value".to_string()),
                data: "test_value".to_string(),
                created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                resume_id: Some(Uuid::nil()),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_cvResumeVersion_dto_from_entity() {
            let entity = create_test_entity();
            let dto = CvResumeVersionDto::from(entity);
            assert_eq!(dto.id, Uuid::nil());
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_cvResumeVersion_dto_serialization() {
            let entity = create_test_entity();
            let dto = CvResumeVersionDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
        }

        #[test]
        fn test_cvResumeVersion_dto_deserialization() {
            let json = r#"{"id":"00000000-0000-0000-0000-000000000000","versionNumber":1,"data":"test"}"#;
            let dto: CvResumeVersionDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, Uuid::nil());
        }
    }

    mod create_cvResumeVersion_dto_tests {
        use super::*;

        #[test]
        fn test_create_cvResumeVersion_dto_valid() {
            let dto = CreateCvResumeVersionDto {
                version_number: 1,
                title: None,
                template: None,
                data: "valid_value".to_string(),
                created_at: None,
                resume_id: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_cvResumeVersion_dto_deserialization() {
            let json = r#"{"versionNumber":1,"data":"test_value"}"#;
            let result: Result<CreateCvResumeVersionDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }

    mod update_cvResumeVersion_dto_tests {
        use super::*;

        #[test]
        fn test_update_cvResumeVersion_dto_all_none_valid() {
            let dto = UpdateCvResumeVersionDto {
                version_number: None,
                title: None,
                template: None,
                data: None,
                created_at: None,
                resume_id: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_cvResumeVersion_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateCvResumeVersionDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
