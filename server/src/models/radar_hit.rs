use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// RadarHit entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::radar_hit)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RadarHit {
    pub id: i32,
    pub user_id: String,
    pub score: Option<f64>,
    pub why_you: Option<String>,
    pub seen: Option<bool>,
    pub dismissed: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<i32>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New RadarHit for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::radar_hit)]
pub struct NewRadarHit {
    pub user_id: String,
    pub score: Option<f64>,
    pub why_you: Option<String>,
    pub seen: Option<bool>,
    pub dismissed: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<i32>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// RadarHit update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::radar_hit)]
pub struct UpdateRadarHit {
    pub user_id: Option<String>,
    pub score: Option<f64>,
    pub why_you: Option<String>,
    pub seen: Option<bool>,
    pub dismissed: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<Option<i32>>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radarHit_clone() {
        let entity = RadarHit {
            id: 1,
            user_id: "test".to_string(),
            score: None,
            why_you: None,
            seen: None,
            dismissed: None,
            created_at: None,
            jobOffer_id: None,
            created_by: Some("system".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let cloned = entity.clone();
        assert_eq!(entity.id, cloned.id);
    }

    #[test]
    fn test_radarHit_debug() {
        let entity = RadarHit {
            id: 1,
            user_id: "test".to_string(),
            score: None,
            why_you: None,
            seen: None,
            dismissed: None,
            created_at: None,
            jobOffer_id: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("RadarHit"));
    }

    #[test]
    fn test_new_radarHit_creation() {
        let new_entity = NewRadarHit {
            user_id: "test".to_string(),
            score: None,
            why_you: None,
            seen: None,
            dismissed: None,
            created_at: None,
            jobOffer_id: None,
            created_by: Some("test_user".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        assert_eq!(new_entity.created_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_update_radarHit_creation() {
        let update = UpdateRadarHit {
            user_id: Some("updated".to_string()),
            score: Some(999.99),
            why_you: Some("updated".to_string()),
            seen: Some(false),
            dismissed: Some(false),
            created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            jobOffer_id: Some(Some(1)),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_radarHit_serialization() {
        let entity = RadarHit {
            id: 1,
            user_id: "test".to_string(),
            score: None,
            why_you: None,
            seen: None,
            dismissed: None,
            created_at: None,
            jobOffer_id: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"id\":1"));
    }
}
