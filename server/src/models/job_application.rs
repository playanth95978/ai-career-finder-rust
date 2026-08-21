use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JobApplication entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::job_application)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct JobApplication {
    pub id: Uuid,
    pub user_id: String,
    pub status: Option<String>,
    pub cover_letter: Option<String>,
    pub notes: Option<String>,
    pub match_score: Option<f64>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub applied_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<Uuid>,
    pub candidateProfile_id: Option<Uuid>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New JobApplication for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::job_application)]
pub struct NewJobApplication {
    pub user_id: String,
    pub status: Option<String>,
    pub cover_letter: Option<String>,
    pub notes: Option<String>,
    pub match_score: Option<f64>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub applied_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<Uuid>,
    pub candidateProfile_id: Option<Uuid>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// JobApplication update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::job_application)]
pub struct UpdateJobApplication {
    pub user_id: Option<String>,
    pub status: Option<String>,
    pub cover_letter: Option<String>,
    pub notes: Option<String>,
    pub match_score: Option<f64>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub applied_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<Option<Uuid>>,
    pub candidateProfile_id: Option<Option<Uuid>>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jobApplication_clone() {
        let entity = JobApplication {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            status: None,
            cover_letter: None,
            notes: None,
            match_score: None,
            created_at: None,
            updated_at: None,
            applied_at: None,
            jobOffer_id: None,
            candidateProfile_id: None,
            created_by: Some("system".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let cloned = entity.clone();
        assert_eq!(entity.id, cloned.id);
    }

    #[test]
    fn test_jobApplication_debug() {
        let entity = JobApplication {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            status: None,
            cover_letter: None,
            notes: None,
            match_score: None,
            created_at: None,
            updated_at: None,
            applied_at: None,
            jobOffer_id: None,
            candidateProfile_id: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("JobApplication"));
    }

    #[test]
    fn test_new_jobApplication_creation() {
        let new_entity = NewJobApplication {
            user_id: "test".to_string(),
            status: None,
            cover_letter: None,
            notes: None,
            match_score: None,
            created_at: None,
            updated_at: None,
            applied_at: None,
            jobOffer_id: None,
            candidateProfile_id: None,
            created_by: Some("test_user".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        assert_eq!(new_entity.created_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_update_jobApplication_creation() {
        let update = UpdateJobApplication {
            user_id: Some("updated".to_string()),
            status: Some("updated".to_string()),
            cover_letter: Some("updated".to_string()),
            notes: Some("updated".to_string()),
            match_score: Some(999.99),
            created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            updated_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            applied_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            jobOffer_id: Some(Some(Uuid::nil())),
            candidateProfile_id: Some(Some(Uuid::nil())),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_jobApplication_serialization() {
        let entity = JobApplication {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            status: None,
            cover_letter: None,
            notes: None,
            match_score: None,
            created_at: None,
            updated_at: None,
            applied_at: None,
            jobOffer_id: None,
            candidateProfile_id: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
    }
}
