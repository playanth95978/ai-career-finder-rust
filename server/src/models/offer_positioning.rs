use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// OfferPositioning entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::offer_positioning)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OfferPositioning {
    pub id: i32,
    pub user_id: String,
    pub result: String,
    pub created_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<i32>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New OfferPositioning for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::offer_positioning)]
pub struct NewOfferPositioning {
    pub user_id: String,
    pub result: String,
    pub created_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<i32>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// OfferPositioning update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::offer_positioning)]
pub struct UpdateOfferPositioning {
    pub user_id: Option<String>,
    pub result: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<Option<i32>>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offerPositioning_clone() {
        let entity = OfferPositioning {
            id: 1,
            user_id: "test".to_string(),
            result: "test".to_string(),
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
    fn test_offerPositioning_debug() {
        let entity = OfferPositioning {
            id: 1,
            user_id: "test".to_string(),
            result: "test".to_string(),
            created_at: None,
            jobOffer_id: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("OfferPositioning"));
    }

    #[test]
    fn test_new_offerPositioning_creation() {
        let new_entity = NewOfferPositioning {
            user_id: "test".to_string(),
            result: "test".to_string(),
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
    fn test_update_offerPositioning_creation() {
        let update = UpdateOfferPositioning {
            user_id: Some("updated".to_string()),
            result: Some("updated".to_string()),
            created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            jobOffer_id: Some(Some(1)),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_offerPositioning_serialization() {
        let entity = OfferPositioning {
            id: 1,
            user_id: "test".to_string(),
            result: "test".to_string(),
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
