use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OfferTailoredResume entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::offer_tailored_resume)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OfferTailoredResume {
    pub id: Uuid,
    pub user_id: String,
    pub data: String,
    pub title: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<Uuid>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New OfferTailoredResume for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = crate::db::schema::offer_tailored_resume)]
pub struct NewOfferTailoredResume {
    pub user_id: String,
    pub data: String,
    pub title: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<Uuid>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// OfferTailoredResume update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = crate::db::schema::offer_tailored_resume)]
pub struct UpdateOfferTailoredResume {
    pub user_id: Option<String>,
    pub data: Option<String>,
    pub title: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub jobOffer_id: Option<Option<Uuid>>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offerTailoredResume_clone() {
        let entity = OfferTailoredResume {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            data: "test".to_string(),
            title: None,
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
    fn test_offerTailoredResume_debug() {
        let entity = OfferTailoredResume {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            data: "test".to_string(),
            title: None,
            created_at: None,
            jobOffer_id: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let debug_str = format!("{:?}", entity);
        assert!(debug_str.contains("OfferTailoredResume"));
    }

    #[test]
    fn test_new_offerTailoredResume_creation() {
        let new_entity = NewOfferTailoredResume {
            user_id: "test".to_string(),
            data: "test".to_string(),
            title: None,
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
    fn test_update_offerTailoredResume_creation() {
        let update = UpdateOfferTailoredResume {
            user_id: Some("updated".to_string()),
            data: Some("updated".to_string()),
            title: Some("updated".to_string()),
            created_at: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            jobOffer_id: Some(Some(Uuid::nil())),
            last_modified_by: Some("updater".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.last_modified_by, Some("updater".to_string()));
    }

    #[test]
    fn test_offerTailoredResume_serialization() {
        let entity = OfferTailoredResume {
            id: Uuid::nil(),
            user_id: "test".to_string(),
            data: "test".to_string(),
            title: None,
            created_at: None,
            jobOffer_id: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"id\":\"00000000-0000-0000-0000-000000000000\""));
    }
}
