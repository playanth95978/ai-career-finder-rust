use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;
use crate::dto::common::deserialize_option_naive_datetime;

use crate::models::JobOffer;
use uuid::Uuid;

/// JobOffer DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobOfferDto {
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reindex_version: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexing_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salary_min: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salary_max: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salary_currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experience_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub published_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub indexed_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub expires_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub last_checked_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,
}

impl From<JobOffer> for JobOfferDto {
    fn from(entity: JobOffer) -> Self {
        Self {
            id: entity.id,
            title: entity.title,
            company: entity.company,
            location: entity.location,
            country: entity.country,
            remote: entity.remote,
            description: entity.description,
            search_text: entity.search_text,
            skills: entity.skills,
            metadata: entity.metadata,
            raw_payload: entity.raw_payload,
            content_hash: entity.content_hash,
            embedding_status: entity.embedding_status,
            embedding_model: entity.embedding_model,
            reindex_version: entity.reindex_version,
            retry_count: entity.retry_count,
            indexing_error: entity.indexing_error,
            source: entity.source,
            source_id: entity.source_id,
            apply_url: entity.apply_url,
            salary_min: entity.salary_min,
            salary_max: entity.salary_max,
            salary_currency: entity.salary_currency,
            contract_type: entity.contract_type,
            experience_level: entity.experience_level,
            category: entity.category,
            source_category: entity.source_category,
            published_at: entity.published_at,
            created_at: entity.created_at,
            indexed_at: entity.indexed_at,
            updated_at: entity.updated_at,
            expires_at: entity.expires_at,
            last_checked_at: entity.last_checked_at,
            created_by: entity.created_by,
            created_date: entity.created_date.map(|d| d.to_string()),
            last_modified_by: entity.last_modified_by,
            last_modified_date: entity.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new JobOffer
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobOfferDto {
    #[validate(length(min = 1))]
    pub title: String,
    pub company: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub remote: Option<bool>,
    pub description: Option<String>,
    pub search_text: Option<String>,
    pub skills: Option<String>,
    pub metadata: Option<String>,
    pub raw_payload: Option<String>,
    #[validate(length(max = 64))]
    pub content_hash: Option<String>,
    pub embedding_status: Option<String>,
    pub embedding_model: Option<String>,
    pub reindex_version: Option<i32>,
    pub retry_count: Option<i32>,
    pub indexing_error: Option<String>,
    pub source: Option<String>,
    pub source_id: Option<String>,
    pub apply_url: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: Option<String>,
    pub contract_type: Option<String>,
    pub experience_level: Option<String>,
    pub category: Option<String>,
    pub source_category: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub published_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub indexed_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub expires_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub last_checked_at: Option<NaiveDateTime>,
}

impl CreateJobOfferDto {
}

/// DTO for updating a JobOffer
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateJobOfferDto {
    pub title: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub remote: Option<bool>,
    pub description: Option<String>,
    pub search_text: Option<String>,
    pub skills: Option<String>,
    pub metadata: Option<String>,
    pub raw_payload: Option<String>,
    #[validate(length(max = 64))]
    pub content_hash: Option<String>,
    pub embedding_status: Option<String>,
    pub embedding_model: Option<String>,
    pub reindex_version: Option<i32>,
    pub retry_count: Option<i32>,
    pub indexing_error: Option<String>,
    pub source: Option<String>,
    pub source_id: Option<String>,
    pub apply_url: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: Option<String>,
    pub contract_type: Option<String>,
    pub experience_level: Option<String>,
    pub category: Option<String>,
    pub source_category: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub published_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub indexed_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub expires_at: Option<NaiveDateTime>,
    #[serde(default, deserialize_with = "deserialize_option_naive_datetime")]
    pub last_checked_at: Option<NaiveDateTime>,
}

impl UpdateJobOfferDto {
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use validator::Validate;

    mod jobOffer_dto_tests {
        use super::*;

        fn create_test_entity() -> JobOffer {
            JobOffer {
                id: Uuid::nil(),
                title: "test_value".to_string(),
                company: Some("test_value".to_string()),
                location: Some("test_value".to_string()),
                country: Some("test_value".to_string()),
                remote: Some(true),
                description: Some("test_value".to_string()),
                search_text: Some("test_value".to_string()),
                skills: Some("test_value".to_string()),
                metadata: Some("test_value".to_string()),
                raw_payload: Some("test_value".to_string()),
                content_hash: Some("test_value".to_string()),
                embedding_status: Some("test_value".to_string()),
                embedding_model: Some("test_value".to_string()),
                reindex_version: Some(42),
                retry_count: Some(42),
                indexing_error: Some("test_value".to_string()),
                source: Some("test_value".to_string()),
                source_id: Some("test_value".to_string()),
                apply_url: Some("test_value".to_string()),
                salary_min: Some(42),
                salary_max: Some(42),
                salary_currency: Some("test_value".to_string()),
                contract_type: Some("test_value".to_string()),
                experience_level: Some("test_value".to_string()),
                category: Some("test_value".to_string()),
                source_category: Some("test_value".to_string()),
                published_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                indexed_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                updated_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                expires_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_checked_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_jobOffer_dto_from_entity() {
            let entity = create_test_entity();
            let dto = JobOfferDto::from(entity);
            assert_eq!(dto.id, Uuid::nil());
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
        }

        #[test]
        fn test_jobOffer_dto_serialization() {
            let entity = create_test_entity();
            let dto = JobOfferDto::from(entity);
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
        }

        #[test]
        fn test_jobOffer_dto_deserialization() {
            let json = r#"{"id":"00000000-0000-0000-0000-000000000000","title":"test"}"#;
            let dto: JobOfferDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, Uuid::nil());
        }
    }

    mod create_jobOffer_dto_tests {
        use super::*;

        #[test]
        fn test_create_jobOffer_dto_valid() {
            let dto = CreateJobOfferDto {
                title: "valid_value".to_string(),
                company: None,
                location: None,
                country: None,
                remote: None,
                description: None,
                search_text: None,
                skills: None,
                metadata: None,
                raw_payload: None,
                content_hash: None,
                embedding_status: None,
                embedding_model: None,
                reindex_version: None,
                retry_count: None,
                indexing_error: None,
                source: None,
                source_id: None,
                apply_url: None,
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                contract_type: None,
                experience_level: None,
                category: None,
                source_category: None,
                published_at: None,
                created_at: None,
                indexed_at: None,
                updated_at: None,
                expires_at: None,
                last_checked_at: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_jobOffer_dto_deserialization() {
            let json = r#"{"title":"test_value"}"#;
            let result: Result<CreateJobOfferDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_jobOffer_dto_empty_string_invalid() {
            let dto = CreateJobOfferDto {
                title: "".to_string(),
                company: None,
                location: None,
                country: None,
                remote: None,
                description: None,
                search_text: None,
                skills: None,
                metadata: None,
                raw_payload: None,
                content_hash: None,
                embedding_status: None,
                embedding_model: None,
                reindex_version: None,
                retry_count: None,
                indexing_error: None,
                source: None,
                source_id: None,
                apply_url: None,
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                contract_type: None,
                experience_level: None,
                category: None,
                source_category: None,
                published_at: None,
                created_at: None,
                indexed_at: None,
                updated_at: None,
                expires_at: None,
                last_checked_at: None,
            };
            assert!(dto.validate().is_err());
        }
    }

    mod update_jobOffer_dto_tests {
        use super::*;

        #[test]
        fn test_update_jobOffer_dto_all_none_valid() {
            let dto = UpdateJobOfferDto {
                title: None,
                company: None,
                location: None,
                country: None,
                remote: None,
                description: None,
                search_text: None,
                skills: None,
                metadata: None,
                raw_payload: None,
                content_hash: None,
                embedding_status: None,
                embedding_model: None,
                reindex_version: None,
                retry_count: None,
                indexing_error: None,
                source: None,
                source_id: None,
                apply_url: None,
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                contract_type: None,
                experience_level: None,
                category: None,
                source_category: None,
                published_at: None,
                created_at: None,
                indexed_at: None,
                updated_at: None,
                expires_at: None,
                last_checked_at: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_jobOffer_dto_deserialization() {
            let json = r#"{}"#;
            let result: Result<UpdateJobOfferDto, _> = serde_json::from_str(json);
            assert!(result.is_ok());
        }
    }
}
