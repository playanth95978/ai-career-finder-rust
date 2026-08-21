use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// CandidateProfile entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::candidate_profile)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CandidateProfile {
    pub id: Uuid,
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
    pub embedded_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New CandidateProfile for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::candidate_profile)]
pub struct NewCandidateProfile {
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
    pub embedded_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// CandidateProfile update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::candidate_profile)]
pub struct UpdateCandidateProfile {
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
    pub embedded_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidateProfile_clone() {
        let entity = CandidateProfile {
            id: Uuid::nil(),
            user_id: "test".to_string(),
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
            created_by: Some("system".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let cloned = entity.clone();
        assert_eq!(entity.id, cloned.id);
    }

    #[test]
    fn test_candidateProfile_debug() {
        let entity = CandidateProfile {
            id: Uuid::nil(),
            user_id: "test".to_string(),
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
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("CandidateProfile"));
    }

    #[test]
    fn test_new_candidateProfile_creation() {
        let new_entity = NewCandidateProfile {
            user_id: "test".to_string(),
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
            created_by: Some("test_user".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        assert_eq!(new_entity.created_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_update_candidateProfile_creation() {
        let update = UpdateCandidateProfile {
            user_id: Some("updated".to_string()),
            full_name: Some("updated".to_string()),
            email: Some("updated".to_string()),
            location: Some("updated".to_string()),
            years_of_experience: Some(999),
            skills: Some("updated".to_string()),
            experiences: Some("updated".to_string()),
            preferred_roles: Some("updated".to_string()),
            languages: Some("updated".to_string()),
            education: Some("updated".to_string()),
            certifications: Some("updated".to_string()),
            raw_markdown: Some("updated".to_string()),
            cv_filename: Some("updated".to_string()),
            embedding_model: Some("updated".to_string()),
            embedded_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            updated_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_candidateProfile_serialization() {
        let entity = CandidateProfile {
            id: Uuid::nil(),
            user_id: "test".to_string(),
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
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
    }
}
