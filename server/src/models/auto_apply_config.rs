use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AutoApplyConfig entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::auto_apply_config)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AutoApplyConfig {
    pub id: Uuid,
    pub user_id: String,
    pub mode: Option<String>,
    pub min_score: Option<f64>,
    pub max_per_day: Option<i32>,
    pub sources: Option<String>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New AutoApplyConfig for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::auto_apply_config)]
pub struct NewAutoApplyConfig {
    pub user_id: String,
    pub mode: Option<String>,
    pub min_score: Option<f64>,
    pub max_per_day: Option<i32>,
    pub sources: Option<String>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// AutoApplyConfig update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::auto_apply_config)]
pub struct UpdateAutoApplyConfig {
    pub user_id: Option<String>,
    pub mode: Option<String>,
    pub min_score: Option<f64>,
    pub max_per_day: Option<i32>,
    pub sources: Option<String>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autoApplyConfig_clone() {
        let entity = AutoApplyConfig {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            mode: None,
            min_score: None,
            max_per_day: None,
            sources: None,
            created_by: Some("system".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let cloned = entity.clone();
        assert_eq!(entity.id, cloned.id);
    }

    #[test]
    fn test_autoApplyConfig_debug() {
        let entity = AutoApplyConfig {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            mode: None,
            min_score: None,
            max_per_day: None,
            sources: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("AutoApplyConfig"));
    }

    #[test]
    fn test_new_autoApplyConfig_creation() {
        let new_entity = NewAutoApplyConfig {
            user_id: "test".to_string(),
            mode: None,
            min_score: None,
            max_per_day: None,
            sources: None,
            created_by: Some("test_user".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        assert_eq!(new_entity.created_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_update_autoApplyConfig_creation() {
        let update = UpdateAutoApplyConfig {
            user_id: Some("updated".to_string()),
            mode: Some("updated".to_string()),
            min_score: Some(999.99),
            max_per_day: Some(999),
            sources: Some("updated".to_string()),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_autoApplyConfig_serialization() {
        let entity = AutoApplyConfig {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            mode: None,
            min_score: None,
            max_per_day: None,
            sources: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
    }
}
