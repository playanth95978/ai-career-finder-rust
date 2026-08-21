use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;
use crate::dto::common::deserialize_option_naive_datetime;
use crate::dto::common::deserialize_optional_relationship;

use crate::models::{OfferPositioning, JobOffer};

/// Minimal DTO for JobOffer relationship in OfferPositioning responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OfferPositioningJobOfferRelationDto {
    pub id: i32,
    pub title: String,
}

impl From<JobOffer> for OfferPositioningJobOfferRelationDto {
    fn from(entity: JobOffer) -> Self {
        Self {
            id: entity.id,
            title: entity.title,
        }
    }
}

/// OfferPositioning DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OfferPositioningDto {
    pub id: i32,
    pub user_id: String,
    pub result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobOffer: Option<OfferPositioningJobOfferRelationDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<OfferPositioning> for OfferPositioningDto {
    fn from(entity: OfferPositioning) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            result: entity.result,
            created_at: entity.created_at,
            jobOffer: None,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

impl OfferPositioningDto {
    /// Create a DTO from entity with related entities
    pub fn from_with_relations(
        entity: OfferPositioning,
        jobOffer: Option<JobOffer>,
    ) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            result: entity.result,
            created_at: entity.created_at,
            jobOffer: jobOffer.map(OfferPositioningJobOfferRelationDto::from),
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new OfferPositioning
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateOfferPositioningDto {
    #[validate(length(min = 1))]
    pub user_id: String,
    pub result: String,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, rename = "jobOffer", deserialize_with = "deserialize_optional_relationship")]
    pub jobOffer_id: Option<i32>,
}

impl CreateOfferPositioningDto {
}

/// DTO for updating a OfferPositioning
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOfferPositioningDto {
    pub user_id: Option<String>,
    pub result: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, rename = "jobOffer", deserialize_with = "deserialize_optional_relationship")]
    pub jobOffer_id: Option<i32>,
}

impl UpdateOfferPositioningDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod offerPositioning_dto_tests {
        use super::*;

        fn create_test_entity() -> OfferPositioning {
            OfferPositioning {
                id: 1,
                user_id: "test_value".to_string(),
                result: "test_value".to_string(),
                created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                jobOffer_id: Some(1),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_offerPositioning_dto_from_entity() {
            let entity = create_test_entity();
            let dto = OfferPositioningDto::from(entity);
            assert_eq!(dto.id, 1);
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_offerPositioning_dto_serialization() {
            let entity = create_test_entity();
            let dto = OfferPositioningDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":1"));
        }

        #[test]
        fn test_offerPositioning_dto_deserialization() {
            let json = r#"{"id":1,"userId":"test","result":"test"}"#;
            let dto: OfferPositioningDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, 1);
        }
    }

    mod create_offerPositioning_dto_tests {
        use super::*;

        #[test]
        fn test_create_offerPositioning_dto_valid() {
            let dto = CreateOfferPositioningDto {
                user_id: "valid_value".to_string(),
                result: "valid_value".to_string(),
                created_at: None,
                jobOffer_id: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_offerPositioning_dto_deserialization() {
            let json = r#"{"userId":"test_value","result":"test_value"}"#;
            let result: Result<CreateOfferPositioningDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_offerPositioning_dto_empty_string_invalid() {
            let dto = CreateOfferPositioningDto {
                user_id: "".to_string(),
                result: "valid".to_string(),
                created_at: None,
                jobOffer_id: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_offerPositioning_dto_tests {
        use super::*;

        #[test]
        fn test_update_offerPositioning_dto_all_none_valid() {
            let dto = UpdateOfferPositioningDto {
                user_id: None,
                result: None,
                created_at: None,
                jobOffer_id: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_offerPositioning_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateOfferPositioningDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
