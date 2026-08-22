use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::User;

/// User DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub login: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub activated: bool,
    /// Le garde d'onboarding du front en depend : tant qu'il est faux, le wizard se rouvre.
    ///
    /// `default` a la deserialisation : le drapeau est decide par le serveur, un client qui
    /// renvoie un `UserDto` sans ce champ ne doit pas se faire rejeter.
    #[serde(default)]
    pub onboarding_completed: bool,
    pub lang_key: Option<String>,
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorities: Option<Vec<String>>,
    pub created_by: Option<String>,
    pub created_date: Option<String>,
    pub last_modified_by: Option<String>,
    pub last_modified_date: Option<String>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: Some(user.id),
            login: user.login,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            activated: user.activated,
            onboarding_completed: user.onboarding_completed,
            lang_key: user.lang_key,
            image_url: user.image_url,
            authorities: None,
            created_by: user.created_by,
            created_date: user.created_date.map(|d| d.to_string()),
            last_modified_by: user.last_modified_by,
            last_modified_date: user.last_modified_date.map(|d| d.to_string()),
        }
    }
}

/// DTO for creating a new user
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserDto {
    #[validate(length(min = 1, max = 50))]
    pub login: String,

    /// Optional. If absent, the server generates a random password and the
    /// user must use the reset-password flow to set their own. Matches the
    /// JHipster admin-create-user convention (Cypress tests POST without
    /// a password — bug surfaced by 1-a.5.0).
    #[validate(length(min = 4, max = 100))]
    pub password: Option<String>,

    #[validate(length(max = 50))]
    pub first_name: Option<String>,

    #[validate(length(max = 50))]
    pub last_name: Option<String>,

    #[validate(email, length(min = 5, max = 254))]
    pub email: String,

    pub activated: Option<bool>,

    #[validate(length(min = 2, max = 10))]
    pub lang_key: Option<String>,

    #[validate(length(max = 256))]
    pub image_url: Option<String>,

    pub authorities: Option<Vec<String>>,
}

/// DTO for updating a user
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserDto {
    /// Login of the user to update (required when updating via PUT /api/admin/users)
    #[validate(length(min = 1, max = 50))]
    pub login: Option<String>,

    #[validate(length(max = 50))]
    pub first_name: Option<String>,

    #[validate(length(max = 50))]
    pub last_name: Option<String>,

    #[validate(email, length(min = 5, max = 254))]
    pub email: Option<String>,

    pub activated: Option<bool>,

    #[validate(length(min = 2, max = 10))]
    pub lang_key: Option<String>,

    #[validate(length(max = 256))]
    pub image_url: Option<String>,

    pub authorities: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    mod user_dto_tests {
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
                onboarding_completed: false,
                lang_key: Some("en".to_string()),
                image_url: Some("https://example.com/avatar.png".to_string()),
                created_by: Some("system".to_string()),
                created_date: Some(NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
                last_modified_by: Some("admin".to_string()),
                last_modified_date: Some(NaiveDateTime::parse_from_str("2024-01-02 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            }
        }

        #[test]
        fn test_user_dto_from_user() {
            let user = create_test_user();
            let dto = UserDto::from(user);
            assert_eq!(dto.id, Some(1));
            assert_eq!(dto.login, "testuser");
            assert_eq!(dto.first_name, Some("Test".to_string()));
            assert_eq!(dto.last_name, Some("User".to_string()));
            assert_eq!(dto.email, "test@example.com");
            assert!(dto.activated);
            assert_eq!(dto.lang_key, Some("en".to_string()));
            assert_eq!(dto.image_url, Some("https://example.com/avatar.png".to_string()));
            assert!(dto.authorities.is_none()); // Not populated from User
            assert_eq!(dto.created_by, Some("system".to_string()));
            assert!(dto.created_date.is_some());
            assert_eq!(dto.last_modified_by, Some("admin".to_string()));
            assert!(dto.last_modified_date.is_some());
        }

        #[test]
        fn test_user_dto_from_user_with_none_dates() {
            let user = User {
                id: 2,
                login: "user2".to_string(),
                password_hash: "$argon2id$hash".to_string(),
                first_name: None,
                last_name: None,
                email: "user2@example.com".to_string(),
                activated: false,
                onboarding_completed: false,
                lang_key: None,
                image_url: None,
                created_by: None,
                created_date: None,
                last_modified_by: None,
                last_modified_date: None,
            };
            let dto = UserDto::from(user);
            assert_eq!(dto.id, Some(2));
            assert!(dto.first_name.is_none());
            assert!(dto.created_date.is_none());
            assert!(dto.last_modified_date.is_none());
        }

        #[test]
        fn test_user_dto_serialization() {
            let dto = UserDto {
                id: Some(1),
                login: "testuser".to_string(),
                first_name: Some("Test".to_string()),
                last_name: Some("User".to_string()),
                email: "test@example.com".to_string(),
                activated: true,
                onboarding_completed: false,
                lang_key: Some("en".to_string()),
                image_url: None,
                authorities: Some(vec!["ROLE_USER".to_string()]),
                created_by: Some("system".to_string()),
                created_date: Some("2024-01-01T00:00:00".to_string()),
                last_modified_by: Some("system".to_string()),
                last_modified_date: Some("2024-01-01T00:00:00".to_string()),
            };
            let json = serde_json::to_string(&dto).unwrap();
            assert!(json.contains("\"login\":\"testuser\""));
            assert!(json.contains("\"firstName\":\"Test\""));
            assert!(json.contains("\"lastName\":\"User\""));
            assert!(json.contains("\"activated\":true"));
        }

        #[test]
        fn test_user_dto_deserialization() {
            let json = r#"{"id":1,"login":"testuser","firstName":"Test","lastName":"User","email":"test@example.com","activated":true,"langKey":"en"}"#;
            let dto: UserDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.id, Some(1));
            assert_eq!(dto.login, "testuser");
            assert_eq!(dto.first_name, Some("Test".to_string()));
        }

        #[test]
        fn test_user_dto_skips_none_authorities() {
            let dto = UserDto {
                id: Some(1),
                login: "testuser".to_string(),
                first_name: None,
                last_name: None,
                email: "test@example.com".to_string(),
                activated: true,
                onboarding_completed: false,
                lang_key: None,
                image_url: None,
                authorities: None,
                created_by: None,
                created_date: None,
                last_modified_by: None,
                last_modified_date: None,
            };
            let json = serde_json::to_string(&dto).unwrap();
            assert!(!json.contains("authorities"));
        }
    }

    mod create_user_dto_tests {
        use super::*;

        #[test]
        fn test_create_user_dto_valid() {
            let dto = CreateUserDto {
                login: "newuser".to_string(),
                password: Some("password123".to_string()),
                first_name: Some("New".to_string()),
                last_name: Some("User".to_string()),
                email: "newuser@example.com".to_string(),
                activated: Some(true),
                lang_key: Some("en".to_string()),
                image_url: None,
                authorities: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_create_user_dto_empty_login_invalid() {
            let dto = CreateUserDto {
                login: "".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "test@example.com".to_string(),
                activated: None,
                lang_key: None,
                image_url: None,
                authorities: None,
            };
            assert!(dto.validate().is_err());
        }

        #[test]
        fn test_create_user_dto_short_password_invalid() {
            let dto = CreateUserDto {
                login: "testuser".to_string(),
                password: Some("abc".to_string()),
                first_name: None,
                last_name: None,
                email: "test@example.com".to_string(),
                activated: None,
                lang_key: None,
                image_url: None,
                authorities: None,
            };
            assert!(dto.validate().is_err());
        }

        #[test]
        fn test_create_user_dto_invalid_email() {
            let dto = CreateUserDto {
                login: "testuser".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "invalid-email".to_string(),
                activated: None,
                lang_key: None,
                image_url: None,
                authorities: None,
            };
            assert!(dto.validate().is_err());
        }

        #[test]
        fn test_create_user_dto_deserialization() {
            let json = r#"{"login":"testuser","password":"password123","email":"test@example.com"}"#;
            let dto: CreateUserDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.login, "testuser");
            assert_eq!(dto.password, Some("password123".to_string()));
            assert_eq!(dto.email, "test@example.com");
        }

        #[test]
        fn test_create_user_dto_password_optional() {
            // Admin-create-user flow: password absent in request body is valid.
            // The handler generates a random one server-side and the user
            // sets their own via reset-password. Bug #13 surfaced by 1-a.5.0
            // gateway-cb cypress against user-management.cy.ts.
            let json = r#"{"login":"newuser","email":"newuser@example.com"}"#;
            let dto: CreateUserDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.login, "newuser");
            assert_eq!(dto.password, None);
            assert!(dto.validate().is_ok());
        }
    }

    mod update_user_dto_tests {
        use super::*;

        #[test]
        fn test_update_user_dto_valid() {
            let dto = UpdateUserDto {
                login: Some("testuser".to_string()),
                first_name: Some("Updated".to_string()),
                last_name: Some("Name".to_string()),
                email: Some("updated@example.com".to_string()),
                activated: Some(true),
                lang_key: Some("fr".to_string()),
                image_url: None,
                authorities: Some(vec!["ROLE_ADMIN".to_string()]),
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_user_dto_all_none_valid() {
            let dto = UpdateUserDto {
                login: None,
                first_name: None,
                last_name: None,
                email: None,
                activated: None,
                lang_key: None,
                image_url: None,
                authorities: None,
            };
            assert!(dto.validate().is_ok());
        }

        #[test]
        fn test_update_user_dto_invalid_email() {
            let dto = UpdateUserDto {
                login: None,
                first_name: None,
                last_name: None,
                email: Some("not-an-email".to_string()),
                activated: None,
                lang_key: None,
                image_url: None,
                authorities: None,
            };
            assert!(dto.validate().is_err());
        }

        #[test]
        fn test_update_user_dto_deserialization() {
            let json = r#"{"login":"testuser","firstName":"Updated","email":"new@example.com"}"#;
            let dto: UpdateUserDto = serde_json::from_str(json).unwrap();
            assert_eq!(dto.login, Some("testuser".to_string()));
            assert_eq!(dto.first_name, Some("Updated".to_string()));
            assert_eq!(dto.email, Some("new@example.com".to_string()));
        }
    }
}
