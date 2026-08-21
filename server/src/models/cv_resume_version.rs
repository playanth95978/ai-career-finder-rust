use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// CvResumeVersion entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::cv_resume_version)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CvResumeVersion {
    pub id: Uuid,
    pub version_number: i32,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: String,
    pub created_at: Option<NaiveDateTime>,
    pub resume_id: Option<Uuid>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New CvResumeVersion for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::cv_resume_version)]
pub struct NewCvResumeVersion {
    pub version_number: i32,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: String,
    pub created_at: Option<NaiveDateTime>,
    pub resume_id: Option<Uuid>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// CvResumeVersion update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::cv_resume_version)]
pub struct UpdateCvResumeVersion {
    pub version_number: Option<i32>,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub resume_id: Option<Option<Uuid>>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cvResumeVersion_clone() {
        let entity = CvResumeVersion {
            id: Uuid::nil(),
            version_number: 42,
            title: None,
            template: None,
            data: "test".to_string(),
            created_at: None,
            resume_id: None,
            created_by: Some("system".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let cloned = entity.clone();
        assert_eq!(entity.id, cloned.id);
    }

    #[test]
    fn test_cvResumeVersion_debug() {
        let entity = CvResumeVersion {
            id: Uuid::nil(),
            version_number: 42,
            title: None,
            template: None,
            data: "test".to_string(),
            created_at: None,
            resume_id: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("CvResumeVersion"));
    }

    #[test]
    fn test_new_cvResumeVersion_creation() {
        let new_entity = NewCvResumeVersion {
            version_number: 42,
            title: None,
            template: None,
            data: "test".to_string(),
            created_at: None,
            resume_id: None,
            created_by: Some("test_user".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        assert_eq!(new_entity.created_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_update_cvResumeVersion_creation() {
        let update = UpdateCvResumeVersion {
            version_number: Some(999),
            title: Some("updated".to_string()),
            template: Some("updated".to_string()),
            data: Some("updated".to_string()),
            created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            resume_id: Some(Some(Uuid::nil())),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_cvResumeVersion_serialization() {
        let entity = CvResumeVersion {
            id: Uuid::nil(),
            version_number: 42,
            title: None,
            template: None,
            data: "test".to_string(),
            created_at: None,
            resume_id: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
    }
}
