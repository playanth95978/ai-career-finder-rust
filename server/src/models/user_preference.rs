use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// UserPreference entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::user_preference)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserPreference {
    pub id: Uuid,
    pub user_id: String,
    pub remote_only: Option<bool>,
    pub contract_type: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub preferred_roles: Option<String>,
    pub excluded_technologies: Option<String>,
    pub preferred_locations: Option<String>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New UserPreference for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::user_preference)]
pub struct NewUserPreference {
    pub user_id: String,
    pub remote_only: Option<bool>,
    pub contract_type: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub preferred_roles: Option<String>,
    pub excluded_technologies: Option<String>,
    pub preferred_locations: Option<String>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// UserPreference update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::user_preference)]
pub struct UpdateUserPreference {
    pub user_id: Option<String>,
    pub remote_only: Option<bool>,
    pub contract_type: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub preferred_roles: Option<String>,
    pub excluded_technologies: Option<String>,
    pub preferred_locations: Option<String>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_userPreference_clone() {
        let entity = UserPreference {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            remote_only: None,
            contract_type: None,
            salary_min: None,
            salary_max: None,
            preferred_roles: None,
            excluded_technologies: None,
            preferred_locations: None,
            created_by: Some("system".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let cloned = entity.clone();
        assert_eq!(entity.id, cloned.id);
    }

    #[test]
    fn test_userPreference_debug() {
        let entity = UserPreference {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            remote_only: None,
            contract_type: None,
            salary_min: None,
            salary_max: None,
            preferred_roles: None,
            excluded_technologies: None,
            preferred_locations: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("UserPreference"));
    }

    #[test]
    fn test_new_userPreference_creation() {
        let new_entity = NewUserPreference {
            user_id: "test".to_string(),
            remote_only: None,
            contract_type: None,
            salary_min: None,
            salary_max: None,
            preferred_roles: None,
            excluded_technologies: None,
            preferred_locations: None,
            created_by: Some("test_user".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        assert_eq!(new_entity.created_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_update_userPreference_creation() {
        let update = UpdateUserPreference {
            user_id: Some("updated".to_string()),
            remote_only: Some(false),
            contract_type: Some("updated".to_string()),
            salary_min: Some(999),
            salary_max: Some(999),
            preferred_roles: Some("updated".to_string()),
            excluded_technologies: Some("updated".to_string()),
            preferred_locations: Some("updated".to_string()),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_userPreference_serialization() {
        let entity = UserPreference {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            remote_only: None,
            contract_type: None,
            salary_min: None,
            salary_max: None,
            preferred_roles: None,
            excluded_technologies: None,
            preferred_locations: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
    }
}
