use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;
use crate::dto::common::deserialize_option_naive_datetime;

use crate::models::CandidateProfile;
use uuid::Uuid;

/// CandidateProfile DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CandidateProfileDto {
    pub id: Uuid,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub years_of_experience: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experiences: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_roles: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub education: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certifications: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_markdown: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cv_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub embedded_at: Option<NaiveDateTime>,
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

impl From<CandidateProfile> for CandidateProfileDto {
    fn from(entity: CandidateProfile) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            full_name: entity.full_name,
            email: entity.email,
            location: entity.location,
            years_of_experience: entity.years_of_experience,
            skills: entity.skills,
            experiences: entity.experiences,
            preferred_roles: entity.preferred_roles,
            languages: entity.languages,
            education: entity.education,
            certifications: entity.certifications,
            raw_markdown: entity.raw_markdown,
            cv_filename: entity.cv_filename,
            embedding_model: entity.embedding_model,
            embedded_at: entity.embedded_at,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new CandidateProfile
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCandidateProfileDto {
    #[validate(length(min = 1))]
    pub user_id: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub location: Option<String>,
    pub years_of_experience: Option<i32>,
    pub skills: Option<String>,
    pub experiences: Option<String>,
    pub preferred_roles: Option<String>,
    pub languages: Option<String>,
    pub education: Option<String>,
    pub certifications: Option<String>,
    pub raw_markdown: Option<String>,
    pub cv_filename: Option<String>,
    pub embedding_model: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub embedded_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}

impl CreateCandidateProfileDto {
}

/// DTO for updating a CandidateProfile
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCandidateProfileDto {
    pub user_id: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub location: Option<String>,
    pub years_of_experience: Option<i32>,
    pub skills: Option<String>,
    pub experiences: Option<String>,
    pub preferred_roles: Option<String>,
    pub languages: Option<String>,
    pub education: Option<String>,
    pub certifications: Option<String>,
    pub raw_markdown: Option<String>,
    pub cv_filename: Option<String>,
    pub embedding_model: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub embedded_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}

impl UpdateCandidateProfileDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod candidateProfile_dto_tests {
        use super::*;

        fn create_test_entity() -> CandidateProfile {
            CandidateProfile {
                id: Uuid::nil(),
                user_id: "test_value".to_string(),
                full_name: Some("test_value".to_string()),
                email: Some("test_value".to_string()),
                location: Some("test_value".to_string()),
                years_of_experience: Some(42),
                skills: Some("test_value".to_string()),
                experiences: Some("test_value".to_string()),
                preferred_roles: Some("test_value".to_string()),
                languages: Some("test_value".to_string()),
                education: Some("test_value".to_string()),
                certifications: Some("test_value".to_string()),
                raw_markdown: Some("test_value".to_string()),
                cv_filename: Some("test_value".to_string()),
                embedding_model: Some("test_value".to_string()),
                embedded_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                updated_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_candidateProfile_dto_from_entity() {
            let entity = create_test_entity();
            let dto = CandidateProfileDto::from(entity);
            assert_eq!(dto.id, Uuid::nil());
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_candidateProfile_dto_serialization() {
            let entity = create_test_entity();
            let dto = CandidateProfileDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
        }

        #[test]
        fn test_candidateProfile_dto_deserialization() {
            let json = r#"{"id":"00000000-0000-0000-0000-000000000000","userId":"test"}"#;
            let dto: CandidateProfileDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, Uuid::nil());
        }
    }

    mod create_candidateProfile_dto_tests {
        use super::*;

        #[test]
        fn test_create_candidateProfile_dto_valid() {
            let dto = CreateCandidateProfileDto {
                user_id: "valid_value".to_string(),
                full_name: None,
                email: None,
                location: None,
                years_of_experience: None,
                skills: None,
                experiences: None,
                preferred_roles: None,
                languages: None,
                education: None,
                certifications: None,
                raw_markdown: None,
                cv_filename: None,
                embedding_model: None,
                embedded_at: None,
                created_at: None,
                updated_at: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_candidateProfile_dto_deserialization() {
            let json = r#"{"userId":"test_value"}"#;
            let result: Result<CreateCandidateProfileDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_candidateProfile_dto_empty_string_invalid() {
            let dto = CreateCandidateProfileDto {
                user_id: "".to_string(),
                full_name: None,
                email: None,
                location: None,
                years_of_experience: None,
                skills: None,
                experiences: None,
                preferred_roles: None,
                languages: None,
                education: None,
                certifications: None,
                raw_markdown: None,
                cv_filename: None,
                embedding_model: None,
                embedded_at: None,
                created_at: None,
                updated_at: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_candidateProfile_dto_tests {
        use super::*;

        #[test]
        fn test_update_candidateProfile_dto_all_none_valid() {
            let dto = UpdateCandidateProfileDto {
                user_id: None,
                full_name: None,
                email: None,
                location: None,
                years_of_experience: None,
                skills: None,
                experiences: None,
                preferred_roles: None,
                languages: None,
                education: None,
                certifications: None,
                raw_markdown: None,
                cv_filename: None,
                embedding_model: None,
                embedded_at: None,
                created_at: None,
                updated_at: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_candidateProfile_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateCandidateProfileDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
