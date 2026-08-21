use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// CvResume entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::cv_resume)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CvResume {
    pub id: i32,
    pub user_id: String,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: String,
    pub version_number: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New CvResume for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::cv_resume)]
pub struct NewCvResume {
    pub user_id: String,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: String,
    pub version_number: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// CvResume update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::cv_resume)]
pub struct UpdateCvResume {
    pub user_id: Option<String>,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: Option<String>,
    pub version_number: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cvResume_clone() {
        let entity = CvResume {
            id: 1,
            user_id: "test".to_string(),
            title: None,
            template: None,
            data: "test".to_string(),
            version_number: 42,
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
    fn test_cvResume_debug() {
        let entity = CvResume {
            id: 1,
            user_id: "test".to_string(),
            title: None,
            template: None,
            data: "test".to_string(),
            version_number: 42,
            created_at: None,
            updated_at: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("CvResume"));
    }

    #[test]
    fn test_new_cvResume_creation() {
        let new_entity = NewCvResume {
            user_id: "test".to_string(),
            title: None,
            template: None,
            data: "test".to_string(),
            version_number: 42,
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
    fn test_update_cvResume_creation() {
        let update = UpdateCvResume {
            user_id: Some("updated".to_string()),
            title: Some("updated".to_string()),
            template: Some("updated".to_string()),
            data: Some("updated".to_string()),
            version_number: Some(999),
            created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            updated_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_cvResume_serialization() {
        let entity = CvResume {
            id: 1,
            user_id: "test".to_string(),
            title: None,
            template: None,
            data: "test".to_string(),
            version_number: 42,
            created_at: None,
            updated_at: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"id\":1"));
    }
}
