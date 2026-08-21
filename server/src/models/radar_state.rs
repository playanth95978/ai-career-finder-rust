use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// RadarState entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::radar_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RadarState {
    pub id: Uuid,
    pub user_id: String,
    pub last_offer_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New RadarState for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::radar_state)]
pub struct NewRadarState {
    pub user_id: String,
    pub last_offer_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// RadarState update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::radar_state)]
pub struct UpdateRadarState {
    pub user_id: Option<String>,
    pub last_offer_at: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radarState_clone() {
        let entity = RadarState {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            last_offer_at: None,
            created_by: Some("system".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let cloned = entity.clone();
        assert_eq!(entity.id, cloned.id);
    }

    #[test]
    fn test_radarState_debug() {
        let entity = RadarState {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            last_offer_at: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("RadarState"));
    }

    #[test]
    fn test_new_radarState_creation() {
        let new_entity = NewRadarState {
            user_id: "test".to_string(),
            last_offer_at: None,
            created_by: Some("test_user".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        assert_eq!(new_entity.created_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_update_radarState_creation() {
        let update = UpdateRadarState {
            user_id: Some("updated".to_string()),
            last_offer_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_radarState_serialization() {
        let entity = RadarState {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            last_offer_at: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
    }
}
