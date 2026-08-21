use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;
use crate::dto::common::deserialize_option_naive_datetime;
use crate::dto::common::deserialize_optional_relationship;

use crate::models::{RadarHit, JobOffer};

/// Minimal DTO for JobOffer relationship in RadarHit responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RadarHitJobOfferRelationDto {
    pub id: i32,
    pub title: String,
}

impl From<JobOffer> for RadarHitJobOfferRelationDto {
    fn from(entity: JobOffer) -> Self {
        Self {
            id: entity.id,
            title: entity.title,
        }
    }
}

/// RadarHit DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RadarHitDto {
    pub id: i32,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why_you: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seen: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobOffer: Option<RadarHitJobOfferRelationDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<RadarHit> for RadarHitDto {
    fn from(entity: RadarHit) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            score: entity.score,
            why_you: entity.why_you,
            seen: entity.seen,
            dismissed: entity.dismissed,
            created_at: entity.created_at,
            jobOffer: None,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

impl RadarHitDto {
    /// Create a DTO from entity with related entities
    pub fn from_with_relations(
        entity: RadarHit,
        jobOffer: Option<JobOffer>,
    ) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            score: entity.score,
            why_you: entity.why_you,
            seen: entity.seen,
            dismissed: entity.dismissed,
            created_at: entity.created_at,
            jobOffer: jobOffer.map(RadarHitJobOfferRelationDto::from),
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new RadarHit
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRadarHitDto {
    #[validate(length(min = 1))]
    pub user_id: String,
    pub score: Option<f64>,
    pub why_you: Option<String>,
    pub seen: Option<bool>,
    pub dismissed: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, rename = "jobOffer", deserialize_with = "deserialize_optional_relationship")]
    pub jobOffer_id: Option<i32>,
}

impl CreateRadarHitDto {
}

/// DTO for updating a RadarHit
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRadarHitDto {
    pub user_id: Option<String>,
    pub score: Option<f64>,
    pub why_you: Option<String>,
    pub seen: Option<bool>,
    pub dismissed: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, rename = "jobOffer", deserialize_with = "deserialize_optional_relationship")]
    pub jobOffer_id: Option<i32>,
}

impl UpdateRadarHitDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod radarHit_dto_tests {
        use super::*;

        fn create_test_entity() -> RadarHit {
            RadarHit {
                id: 1,
                user_id: "test_value".to_string(),
                score: Some(42.5),
                why_you: Some("test_value".to_string()),
                seen: Some(true),
                dismissed: Some(true),
                created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                jobOffer_id: Some(1),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_radarHit_dto_from_entity() {
            let entity = create_test_entity();
            let dto = RadarHitDto::from(entity);
            assert_eq!(dto.id, 1);
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_radarHit_dto_serialization() {
            let entity = create_test_entity();
            let dto = RadarHitDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":1"));
        }

        #[test]
        fn test_radarHit_dto_deserialization() {
            let json = r#"{"id":1,"userId":"test"}"#;
            let dto: RadarHitDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, 1);
        }
    }

    mod create_radarHit_dto_tests {
        use super::*;

        #[test]
        fn test_create_radarHit_dto_valid() {
            let dto = CreateRadarHitDto {
                user_id: "valid_value".to_string(),
                score: None,
                why_you: None,
                seen: None,
                dismissed: None,
                created_at: None,
                jobOffer_id: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_radarHit_dto_deserialization() {
            let json = r#"{"userId":"test_value"}"#;
            let result: Result<CreateRadarHitDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_radarHit_dto_empty_string_invalid() {
            let dto = CreateRadarHitDto {
                user_id: "".to_string(),
                score: None,
                why_you: None,
                seen: None,
                dismissed: None,
                created_at: None,
                jobOffer_id: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_radarHit_dto_tests {
        use super::*;

        #[test]
        fn test_update_radarHit_dto_all_none_valid() {
            let dto = UpdateRadarHitDto {
                user_id: None,
                score: None,
                why_you: None,
                seen: None,
                dismissed: None,
                created_at: None,
                jobOffer_id: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_radarHit_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateRadarHitDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
