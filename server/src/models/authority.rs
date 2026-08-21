use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::schema::authorities;

/// Authority/Role entity
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = authorities)]
#[diesel(primary_key(name))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Authority {
    pub name: String,
}

/// Role types used in the application
pub struct RoleType;

impl RoleType {
    pub const ADMIN: &'static str = "ROLE_ADMIN";
    pub const USER: &'static str = "ROLE_USER";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_type_admin_constant() {
        assert_eq!(RoleType::ADMIN, "ROLE_ADMIN");
    }

    #[test]
    fn test_role_type_user_constant() {
        assert_eq!(RoleType::USER, "ROLE_USER");
    }

    #[test]
    fn test_authority_creation() {
        let authority = Authority {
            name: "ROLE_ADMIN".to_string(),
        };
        assert_eq!(authority.name, "ROLE_ADMIN");
    }

    #[test]
    fn test_authority_clone() {
        let authority = Authority {
            name: "ROLE_USER".to_string(),
        };
        let cloned = authority.clone();
        assert_eq!(cloned.name, authority.name);
    }

    #[test]
    fn test_authority_debug() {
        let authority = Authority {
            name: "ROLE_USER".to_string(),
        };
        let debug_str = format!("{:?}", authority);
        assert!(debug_str.contains("ROLE_USER"));
    }

    #[test]
    fn test_authority_serialization() {
        let authority = Authority {
            name: "ROLE_ADMIN".to_string(),
        };
        let json = serde_json::to_string(&authority).unwrap();
        assert!(json.contains("ROLE_ADMIN"));
    }

    #[test]
    fn test_authority_deserialization() {
        let json = r#"{"name":"ROLE_USER"}"#;
        let authority: Authority = serde_json::from_str(json).unwrap();
        assert_eq!(authority.name, "ROLE_USER");
    }
}
