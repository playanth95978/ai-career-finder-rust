use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::schema::{users, user_authorities};

/// User entity - represents a user in the system
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub login: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub activated: bool,
    pub lang_key: Option<String>,
    pub image_url: Option<String>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// New user for insertion
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub login: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub activated: bool,
    pub lang_key: Option<String>,
    pub image_url: Option<String>,
    pub created_by: Option<String>,
    pub created_date: Option<NaiveDateTime>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// User update changeset
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub activated: Option<bool>,
    pub lang_key: Option<String>,
    pub image_url: Option<String>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<NaiveDateTime>,
}

/// User authority join table
#[derive(Debug, Clone, Queryable, Selectable, Insertable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = user_authorities)]
#[diesel(primary_key(user_id, authority_name))]
pub struct UserAuthority {
    pub user_id: i32,
    pub authority_name: String,
}

impl User {
    /// Check if user has a specific authority/role
    pub fn has_authority(&self, authorities: &[String], authority: &str) -> bool {
        authorities.iter().any(|a| a == authority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    fn create_test_user() -> User {
        User {
            id: 1,
            login: "testuser".to_string(),
            password_hash: "$argon2id$hash".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            email: "test@example.com".to_string(),
            activated: true,
            lang_key: Some("en".to_string()),
            image_url: None,
            created_by: Some("system".to_string()),
            created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            last_modified_by: Some("system".to_string()),
            last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
        }
    }

    #[test]
    fn test_user_has_authority_true() {
        let user = create_test_user();
        let authorities = vec!["ROLE_USER".to_string(), "ROLE_ADMIN".to_string()];
        assert!(user.has_authority(&authorities, "ROLE_USER"));
        assert!(user.has_authority(&authorities, "ROLE_ADMIN"));
    }

    #[test]
    fn test_user_has_authority_false() {
        let user = create_test_user();
        let authorities = vec!["ROLE_USER".to_string()];
        assert!(!user.has_authority(&authorities, "ROLE_ADMIN"));
    }

    #[test]
    fn test_user_has_authority_empty_list() {
        let user = create_test_user();
        let authorities: Vec<String> = vec![];
        assert!(!user.has_authority(&authorities, "ROLE_USER"));
    }

    #[test]
    fn test_user_clone() {
        let user = create_test_user();
        let cloned = user.clone();
        assert_eq!(cloned.id, user.id);
        assert_eq!(cloned.login, user.login);
        assert_eq!(cloned.email, user.email);
    }

    #[test]
    fn test_user_debug() {
        let user = create_test_user();
        let debug_str = format!("{:?}", user);
        assert!(debug_str.contains("testuser"));
        assert!(debug_str.contains("test@example.com"));
    }

    #[test]
    fn test_user_serialization_skips_password() {
        let user = create_test_user();
        let json = serde_json::to_string(&user).unwrap();
        assert!(!json.contains("password_hash"));
        assert!(json.contains("login"));
        assert!(json.contains("email"));
    }

    #[test]
    fn test_new_user_creation() {
        let new_user = NewUser {
            login: "newuser".to_string(),
            password_hash: "$argon2id$newhash".to_string(),
            first_name: Some("New".to_string()),
            last_name: Some("User".to_string()),
            email: "new@example.com".to_string(),
            activated: false,
            lang_key: Some("en".to_string()),
            image_url: None,
            created_by: Some("admin".to_string()),
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        };
        assert_eq!(new_user.login, "newuser");
        assert!(!new_user.activated);
    }

    #[test]
    fn test_update_user_creation() {
        let update = UpdateUser {
            first_name: Some("Updated".to_string()),
            last_name: None,
            email: Some("updated@example.com".to_string()),
            activated: Some(true),
            lang_key: None,
            image_url: None,
            last_modified_by: Some("admin".to_string()),
            last_modified_date: None,
        };
        assert_eq!(update.first_name, Some("Updated".to_string()));
        assert!(update.last_name.is_none());
    }

    #[test]
    fn test_user_authority_creation() {
        let authority = UserAuthority {
            user_id: 1,
            authority_name: "ROLE_USER".to_string(),
        };
        assert_eq!(authority.user_id, 1);
        assert_eq!(authority.authority_name, "ROLE_USER");
    }

}
