use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;
use crate::dto::common::deserialize_option_naive_datetime;
use crate::dto::common::deserialize_optional_relationship;

use crate::models::{JobApplication, JobOffer, CandidateProfile};

/// Minimal DTO for JobOffer relationship in JobApplication responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobApplicationJobOfferRelationDto {
    pub id: i32,
    pub title: String,
}

impl From<JobOffer> for JobApplicationJobOfferRelationDto {
    fn from(entity: JobOffer) -> Self {
        Self {
            id: entity.id,
            title: entity.title,
        }
    }
}

/// Minimal DTO for CandidateProfile relationship in JobApplication responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobApplicationCandidateProfileRelationDto {
    pub id: i32,
}

impl From<CandidateProfile> for JobApplicationCandidateProfileRelationDto {
    fn from(entity: CandidateProfile) -> Self {
        Self {
            id: entity.id,
        }
    }
}

/// JobApplication DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobApplicationDto {
    pub id: i32,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_letter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub applied_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobOffer: Option<JobApplicationJobOfferRelationDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidateProfile: Option<JobApplicationCandidateProfileRelationDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<JobApplication> for JobApplicationDto {
    fn from(entity: JobApplication) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            status: entity.status,
            cover_letter: entity.cover_letter,
            notes: entity.notes,
            match_score: entity.match_score,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            applied_at: entity.applied_at,
            jobOffer: None,
            candidateProfile: None,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

impl JobApplicationDto {
    /// Create a DTO from entity with related entities
    pub fn from_with_relations(
        entity: JobApplication,
        jobOffer: Option<JobOffer>,
        candidateProfile: Option<CandidateProfile>,
    ) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            status: entity.status,
            cover_letter: entity.cover_letter,
            notes: entity.notes,
            match_score: entity.match_score,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            applied_at: entity.applied_at,
            jobOffer: jobOffer.map(JobApplicationJobOfferRelationDto::from),
            candidateProfile: candidateProfile.map(JobApplicationCandidateProfileRelationDto::from),
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new JobApplication
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobApplicationDto {
    #[validate(length(min = 1))]
    pub user_id: String,
    pub status: Option<String>,
    pub cover_letter: Option<String>,
    pub notes: Option<String>,
    pub match_score: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub applied_at: Option<NaiveDateTime>,
    #[serde(default, rename = "jobOffer", deserialize_with = "deserialize_optional_relationship")]
    pub jobOffer_id: Option<i32>,
    #[serde(default, rename = "candidateProfile", deserialize_with = "deserialize_optional_relationship")]
    pub candidateProfile_id: Option<i32>,
}

impl CreateJobApplicationDto {
}

/// DTO for updating a JobApplication
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateJobApplicationDto {
    pub user_id: Option<String>,
    pub status: Option<String>,
    pub cover_letter: Option<String>,
    pub notes: Option<String>,
    pub match_score: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub applied_at: Option<NaiveDateTime>,
    #[serde(default, rename = "jobOffer", deserialize_with = "deserialize_optional_relationship")]
    pub jobOffer_id: Option<i32>,
    #[serde(default, rename = "candidateProfile", deserialize_with = "deserialize_optional_relationship")]
    pub candidateProfile_id: Option<i32>,
}

impl UpdateJobApplicationDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod jobApplication_dto_tests {
        use super::*;

        fn create_test_entity() -> JobApplication {
            JobApplication {
                id: 1,
                user_id: "test_value".to_string(),
                status: Some("test_value".to_string()),
                cover_letter: Some("test_value".to_string()),
                notes: Some("test_value".to_string()),
                match_score: Some(42.5),
                created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                updated_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                applied_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                jobOffer_id: Some(1),
                candidateProfile_id: Some(1),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_jobApplication_dto_from_entity() {
            let entity = create_test_entity();
            let dto = JobApplicationDto::from(entity);
            assert_eq!(dto.id, 1);
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_jobApplication_dto_serialization() {
            let entity = create_test_entity();
            let dto = JobApplicationDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":1"));
        }

        #[test]
        fn test_jobApplication_dto_deserialization() {
            let json = r#"{"id":1,"userId":"test"}"#;
            let dto: JobApplicationDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, 1);
        }
    }

    mod create_jobApplication_dto_tests {
        use super::*;

        #[test]
        fn test_create_jobApplication_dto_valid() {
            let dto = CreateJobApplicationDto {
                user_id: "valid_value".to_string(),
                status: None,
                cover_letter: None,
                notes: None,
                match_score: None,
                created_at: None,
                updated_at: None,
                applied_at: None,
                jobOffer_id: None,
                candidateProfile_id: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_jobApplication_dto_deserialization() {
            let json = r#"{"userId":"test_value"}"#;
            let result: Result<CreateJobApplicationDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_jobApplication_dto_empty_string_invalid() {
            let dto = CreateJobApplicationDto {
                user_id: "".to_string(),
                status: None,
                cover_letter: None,
                notes: None,
                match_score: None,
                created_at: None,
                updated_at: None,
                applied_at: None,
                jobOffer_id: None,
                candidateProfile_id: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_jobApplication_dto_tests {
        use super::*;

        #[test]
        fn test_update_jobApplication_dto_all_none_valid() {
            let dto = UpdateJobApplicationDto {
                user_id: None,
                status: None,
                cover_letter: None,
                notes: None,
                match_score: None,
                created_at: None,
                updated_at: None,
                applied_at: None,
                jobOffer_id: None,
                candidateProfile_id: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_jobApplication_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateJobApplicationDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
