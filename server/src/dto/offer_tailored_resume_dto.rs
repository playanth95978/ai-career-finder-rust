use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;
use crate::dto::common::deserialize_option_naive_datetime;
use crate::dto::common::deserialize_optional_relationship;

use crate::models::{OfferTailoredResume, JobOffer};
use uuid::Uuid;

/// Minimal DTO for JobOffer relationship in OfferTailoredResume responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OfferTailoredResumeJobOfferRelationDto {
    pub id: Uuid,
    pub title: String,
}

impl From<JobOffer> for OfferTailoredResumeJobOfferRelationDto {
    fn from(entity: JobOffer) -> Self {
        Self {
            id: entity.id,
            title: entity.title,
        }
    }
}

/// OfferTailoredResume DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OfferTailoredResumeDto {
    pub id: Uuid,
    pub user_id: String,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobOffer: Option<OfferTailoredResumeJobOfferRelationDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<OfferTailoredResume> for OfferTailoredResumeDto {
    fn from(entity: OfferTailoredResume) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            data: entity.data,
            title: entity.title,
            created_at: entity.created_at,
            jobOffer: None,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

impl OfferTailoredResumeDto {
    /// Create a DTO from entity with related entities
    pub fn from_with_relations(
        entity: OfferTailoredResume,
        jobOffer: Option<JobOffer>,
    ) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            data: entity.data,
            title: entity.title,
            created_at: entity.created_at,
            jobOffer: jobOffer.map(OfferTailoredResumeJobOfferRelationDto::from),
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new OfferTailoredResume
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateOfferTailoredResumeDto {
    #[validate(length(min = 1))]
    pub user_id: String,
    pub data: String,
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, rename = "jobOffer", deserialize_with = "deserialize_optional_relationship")]
    pub jobOffer_id: Option<Uuid>,
}

impl CreateOfferTailoredResumeDto {
}

/// DTO for updating a OfferTailoredResume
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOfferTailoredResumeDto {
    pub user_id: Option<String>,
    pub data: Option<String>,
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, rename = "jobOffer", deserialize_with = "deserialize_optional_relationship")]
    pub jobOffer_id: Option<Uuid>,
}

impl UpdateOfferTailoredResumeDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod offerTailoredResume_dto_tests {
        use super::*;

        fn create_test_entity() -> OfferTailoredResume {
            OfferTailoredResume {
                id: Uuid::nil(),
                user_id: "test_value".to_string(),
                data: "test_value".to_string(),
                title: Some("test_value".to_string()),
                created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                jobOffer_id: Some(Uuid::nil()),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_offerTailoredResume_dto_from_entity() {
            let entity = create_test_entity();
            let dto = OfferTailoredResumeDto::from(entity);
            assert_eq!(dto.id, Uuid::nil());
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_offerTailoredResume_dto_serialization() {
            let entity = create_test_entity();
            let dto = OfferTailoredResumeDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
        }

        #[test]
        fn test_offerTailoredResume_dto_deserialization() {
            let json = r#"{"id":"00000000-0000-0000-0000-000000000000","userId":"test","data":"test"}"#;
            let dto: OfferTailoredResumeDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, Uuid::nil());
        }
    }

    mod create_offerTailoredResume_dto_tests {
        use super::*;

        #[test]
        fn test_create_offerTailoredResume_dto_valid() {
            let dto = CreateOfferTailoredResumeDto {
                user_id: "valid_value".to_string(),
                data: "valid_value".to_string(),
                title: None,
                created_at: None,
                jobOffer_id: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_offerTailoredResume_dto_deserialization() {
            let json = r#"{"userId":"test_value","data":"test_value"}"#;
            let result: Result<CreateOfferTailoredResumeDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_offerTailoredResume_dto_empty_string_invalid() {
            let dto = CreateOfferTailoredResumeDto {
                user_id: "".to_string(),
                data: "valid".to_string(),
                title: None,
                created_at: None,
                jobOffer_id: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_offerTailoredResume_dto_tests {
        use super::*;

        #[test]
        fn test_update_offerTailoredResume_dto_all_none_valid() {
            let dto = UpdateOfferTailoredResumeDto {
                user_id: None,
                data: None,
                title: None,
                created_at: None,
                jobOffer_id: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_offerTailoredResume_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateOfferTailoredResumeDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
