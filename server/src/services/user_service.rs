use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::rngs::OsRng;
use chrono::Utc;
use diesel::prelude::*;

use crate::db::connection::DbConnection;
use crate::db::schema::{authorities, user_authorities, users};
use crate::dto::{CreateUserDto, PageRequest, UpdateUserDto};
use crate::errors::AppError;
use crate::models::{Authority, NewUser, RoleType, UpdateUser, User, UserAuthority};

pub struct UserService;

impl UserService {
    /// Find all users with pagination
    pub fn find_all(
        conn: &mut DbConnection,
        page_request: &PageRequest,
    ) -> Result<(Vec<User>, i64), AppError> {
        let page = page_request.page.unwrap_or(0);
        let size = page_request.size.unwrap_or(20).min(100);
        let offset = page * size;

        let total: i64 = users::table
            .count()
            .get_result(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Get primary sort parameter (format: "field,direction" e.g., "login,asc" or "email,desc")
        let (sort_field, sort_dir) = page_request.primary_sort().unwrap_or(("id", "asc"));

        let is_desc = sort_dir.eq_ignore_ascii_case("desc");

        // Dynamic sorting based on field name
        let results = match sort_field {
            "login" => {
                if is_desc {
                    users::table
                        .order(users::login.desc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                } else {
                    users::table
                        .order(users::login.asc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                }
            }
            "email" => {
                if is_desc {
                    users::table
                        .order(users::email.desc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                } else {
                    users::table
                        .order(users::email.asc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                }
            }
            "firstName" | "first_name" => {
                if is_desc {
                    users::table
                        .order(users::first_name.desc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                } else {
                    users::table
                        .order(users::first_name.asc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                }
            }
            "lastName" | "last_name" => {
                if is_desc {
                    users::table
                        .order(users::last_name.desc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                } else {
                    users::table
                        .order(users::last_name.asc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                }
            }
            "langKey" | "lang_key" => {
                if is_desc {
                    users::table
                        .order(users::lang_key.desc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                } else {
                    users::table
                        .order(users::lang_key.asc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                }
            }
            "activated" => {
                if is_desc {
                    users::table
                        .order(users::activated.desc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                } else {
                    users::table
                        .order(users::activated.asc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                }
            }
            "createdDate" | "created_date" => {
                if is_desc {
                    users::table
                        .order(users::created_date.desc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                } else {
                    users::table
                        .order(users::created_date.asc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                }
            }
            "lastModifiedDate" | "last_modified_date" => {
                if is_desc {
                    users::table
                        .order(users::last_modified_date.desc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                } else {
                    users::table
                        .order(users::last_modified_date.asc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                }
            }
            _ => {
                // Default: sort by id
                if is_desc {
                    users::table
                        .order(users::id.desc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                } else {
                    users::table
                        .order(users::id.asc())
                        .limit(size)
                        .offset(offset)
                        .load::<User>(conn)
                }
            }
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((results, total))
    }

    /// Find user by ID
    pub fn find_by_id(conn: &mut DbConnection, id: i32) -> Result<User, AppError> {
        users::table
            .find(id)
            .first::<User>(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::NotFound(format!("User {} not found", id)),
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Find user by login
    pub fn find_by_login(conn: &mut DbConnection, login: &str) -> Result<User, AppError> {
        users::table
            .filter(users::login.eq(login))
            .first::<User>(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound(format!("User {} not found", login))
                }
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Find user by email
    pub fn find_by_email(conn: &mut DbConnection, email: &str) -> Result<User, AppError> {
        users::table
            .filter(users::email.eq(email))
            .first::<User>(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound(format!("User with email {} not found", email))
                }
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Get authorities for a user
    pub fn get_authorities(conn: &mut DbConnection, user_id: i32) -> Result<Vec<String>, AppError> {
        user_authorities::table
            .filter(user_authorities::user_id.eq(user_id))
            .select(user_authorities::authority_name)
            .load::<String>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    /// Create a new user
    pub fn create(
        conn: &mut DbConnection,
        dto: CreateUserDto,
        created_by: &str,
    ) -> Result<User, AppError> {
        // Check if login already exists
        if users::table
            .filter(users::login.eq(&dto.login))
            .first::<User>(conn)
            .is_ok()
        {
            return Err(AppError::BadRequest("Login already exists".to_string()));
        }

        // Check if email already exists
        if users::table
            .filter(users::email.eq(&dto.email))
            .first::<User>(conn)
            .is_ok()
        {
            return Err(AppError::BadRequest("Email already exists".to_string()));
        }

        // Hash the password. JHipster admin-create-user flow allows omitting
        // the password — the server generates a random one and the user must
        // use the reset-password flow to set their own. Bug #13 surfaced by
        // 1-a.5.0 gateway-cb cypress (user-management.cy.ts POSTs login+email
        // only). The UUID-based fallback is unguessable, so the created user
        // can't log in until they reset.
        let password = dto.password.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let password_hash = Self::hash_password(&password)?;
        let now = Utc::now().naive_utc();

        let new_user = NewUser {
            login: dto.login,
            password_hash,
            first_name: dto.first_name,
            last_name: dto.last_name,
            email: dto.email,
            activated: dto.activated.unwrap_or(false),
            onboarding_completed: false,
            lang_key: dto.lang_key,
            image_url: dto.image_url,
            created_by: Some(created_by.to_string()),
            created_date: Some(now),
            last_modified_by: Some(created_by.to_string()),
            last_modified_date: Some(now),
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // SQLite doesn't support RETURNING, so we fetch the inserted user by login
        let user = users::table
            .filter(users::login.eq(&new_user.login))
            .first::<User>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Add default role
        let authorities = dto.authorities.unwrap_or_else(|| vec![RoleType::USER.to_string()]);
        for authority in authorities {
            diesel::insert_into(user_authorities::table)
                .values(UserAuthority {
                    user_id: user.id,
                    authority_name: authority,
                })
                .execute(conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        Ok(user)
    }

    /// Update an existing user
    pub fn update(
        conn: &mut DbConnection,
        id: i32,
        dto: UpdateUserDto,
        modified_by: &str,
    ) -> Result<User, AppError> {
        let now = Utc::now().naive_utc();

        let update = UpdateUser {
            first_name: dto.first_name,
            last_name: dto.last_name,
            email: dto.email,
            activated: dto.activated,
            onboarding_completed: None,
            lang_key: dto.lang_key,
            image_url: dto.image_url,
            last_modified_by: Some(modified_by.to_string()),
            last_modified_date: Some(now),
        };

        diesel::update(users::table.find(id))
            .set(&update)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Update authorities if provided
        if let Some(authorities) = dto.authorities {
            // Remove existing authorities
            diesel::delete(user_authorities::table.filter(user_authorities::user_id.eq(id)))
                .execute(conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;

            // Add new authorities
            for authority in authorities {
                diesel::insert_into(user_authorities::table)
                    .values(UserAuthority {
                        user_id: id,
                        authority_name: authority,
                    })
                    .execute(conn)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
        }

        Self::find_by_id(conn, id)
    }

    /// Delete a user
    pub fn delete(conn: &mut DbConnection, id: i32) -> Result<(), AppError> {
        // Delete user authorities first
        diesel::delete(user_authorities::table.filter(user_authorities::user_id.eq(id)))
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Delete user
        diesel::delete(users::table.find(id))
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    // Authority CRUD (bug #14 fix from 1-a.5.0). Previously the GET
    // /api/authorities endpoint returned a hard-coded ROLE_ADMIN/ROLE_USER
    // vec; POST returned 405 and the cypress entity/authority.cy.ts test
    // failed. These methods back the endpoints with the existing
    // `authorities` table.

    /// List all authority names (ordered for stable diffs/tests)
    pub fn find_all_authorities(conn: &mut DbConnection) -> Result<Vec<String>, AppError> {
        authorities::table
            .order(authorities::name.asc())
            .select(authorities::name)
            .load::<String>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    /// Create a new authority. Returns BadRequest if it already exists
    /// (the PK is `name`, so a duplicate insert fails with UniqueViolation).
    pub fn create_authority(conn: &mut DbConnection, name: &str) -> Result<Authority, AppError> {
        let authority = Authority { name: name.to_string() };
        diesel::insert_into(authorities::table)
            .values(&authority)
            .execute(conn)
            .map_err(|e| {
                let msg = e.to_string().to_lowercase();
                if msg.contains("unique") || msg.contains("duplicate") {
                    AppError::BadRequest(format!("Authority '{}' already exists", name))
                } else {
                    AppError::Internal(e.to_string())
                }
            })?;
        Ok(authority)
    }

    /// Delete an authority. Returns NotFound if the row didn't exist. The
    /// user_authorities FK is ON DELETE-restricted (no cascade), so deleting
    /// an authority that's still assigned to a user surfaces an FK error
    /// — caller sees BadRequest.
    pub fn delete_authority(conn: &mut DbConnection, name: &str) -> Result<(), AppError> {
        let rows = diesel::delete(authorities::table.find(name.to_string()))
            .execute(conn)
            .map_err(|e| {
                let msg = e.to_string().to_lowercase();
                if msg.contains("foreign key") || msg.contains("references") {
                    AppError::BadRequest(format!("Authority '{}' is still assigned to users", name))
                } else {
                    AppError::Internal(e.to_string())
                }
            })?;
        if rows == 0 {
            return Err(AppError::NotFound(format!("Authority '{}' not found", name)));
        }
        Ok(())
    }

    /// Hash a password using Argon2
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))
    }

    /// Update user's password
    pub fn update_password(
        conn: &mut DbConnection,
        login: &str,
        password_hash: &str,
    ) -> Result<(), AppError> {
        diesel::update(users::table.filter(users::login.eq(login)))
            .set(users::password_hash.eq(password_hash))
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Update current user's account settings (used by POST /api/account)
    pub fn update_account(
        conn: &mut DbConnection,
        login: &str,
        first_name: Option<String>,
        last_name: Option<String>,
        email: String,
        lang_key: Option<String>,
        image_url: Option<String>,
    ) -> Result<(), AppError> {
        let now = Utc::now().naive_utc();

        diesel::update(users::table.filter(users::login.eq(login)))
            .set((
                users::first_name.eq(first_name),
                users::last_name.eq(last_name),
                users::email.eq(email),
                users::lang_key.eq(lang_key),
                users::image_url.eq(image_url),
                users::last_modified_by.eq(login),
                users::last_modified_date.eq(now),
            ))
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Marque l'onboarding de premiere connexion comme termine.
    ///
    /// Le garde du front relit `GET /api/account` juste apres : si le drapeau n'etait pas persiste,
    /// il renverrait l'utilisateur au wizard en boucle.
    pub fn complete_onboarding(conn: &mut DbConnection, login: &str) -> Result<(), AppError> {
        let now = Utc::now().naive_utc();

        let updated = diesel::update(users::table.filter(users::login.eq(login)))
            .set((
                users::onboarding_completed.eq(true),
                users::last_modified_by.eq(login),
                users::last_modified_date.eq(now),
            ))
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if updated == 0 {
            return Err(AppError::NotFound(format!("User {login} not found")));
        }
        Ok(())
    }

    /// Create a user with authorities (used for testing and seeding)
    pub fn create_with_authorities(
        conn: &mut DbConnection,
        new_user: NewUser,
        authorities: Vec<String>,
    ) -> Result<User, AppError> {
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // SQLite doesn't support RETURNING, so we fetch the inserted user by login
        let user = users::table
            .filter(users::login.eq(&new_user.login))
            .first::<User>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Add authorities
        for authority in authorities {
            diesel::insert_into(user_authorities::table)
                .values(UserAuthority {
                    user_id: user.id,
                    authority_name: authority,
                })
                .execute(conn)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        Ok(user)
    }

    /// Create a new registered user (already activated, used when email is disabled)
    pub fn create_registered_user(
        conn: &mut DbConnection,
        login: String,
        email: String,
        password: &str,
        lang_key: Option<String>,
    ) -> Result<User, AppError> {
        // Check if login already exists
        if users::table
            .filter(users::login.eq(&login))
            .first::<User>(conn)
            .is_ok()
        {
            return Err(AppError::BadRequest("Login already exists".to_string()));
        }

        // Check if email already exists
        if users::table
            .filter(users::email.eq(&email))
            .first::<User>(conn)
            .is_ok()
        {
            return Err(AppError::BadRequest("Email already exists".to_string()));
        }

        // Hash the password
        let password_hash = Self::hash_password(password)?;
        let now = Utc::now().naive_utc();

        let new_user = NewUser {
            login: login.to_lowercase(),
            password_hash,
            first_name: None,
            last_name: None,
            email: email.to_lowercase(),
            activated: true,  // Already activated since email is disabled
            onboarding_completed: false,
            lang_key,
            image_url: None,
            created_by: Some("anonymousUser".to_string()),
            created_date: Some(now),
            last_modified_by: Some("anonymousUser".to_string()),
            last_modified_date: Some(now),
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Fetch the inserted user
        let user = users::table
            .filter(users::login.eq(&new_user.login))
            .first::<User>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Add default ROLE_USER
        diesel::insert_into(user_authorities::table)
            .values(UserAuthority {
                user_id: user.id,
                authority_name: RoleType::USER.to_string(),
            })
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(user)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    mod unit_tests {
        use super::*;

        #[test]
        fn test_hash_password_success() {
            let password = "my_secure_password";
            let result = UserService::hash_password(password);
            assert!(result.is_ok());
            let hash = result.unwrap();
            assert!(!hash.is_empty());
            // Argon2 hashes start with $argon2
            assert!(hash.starts_with("$argon2"));
        }

        #[test]
        fn test_hash_password_different_outputs() {
            let password = "same_password";
            let hash1 = UserService::hash_password(password).unwrap();
            let hash2 = UserService::hash_password(password).unwrap();
            // Due to random salt, same password should produce different hashes
            assert_ne!(hash1, hash2);
        }

        #[test]
        fn test_hash_password_empty_string() {
            let result = UserService::hash_password("");
            // Empty password should still be hashable
            assert!(result.is_ok());
        }

        #[test]
        fn test_hash_password_long_password() {
            let password = "a".repeat(1000);
            let result = UserService::hash_password(&password);
            assert!(result.is_ok());
        }

        #[test]
        fn test_hash_password_special_characters() {
            let password = "p@$$w0rd!#$%^&*()";
            let result = UserService::hash_password(password);
            assert!(result.is_ok());
        }

        #[test]
        fn test_hash_password_unicode() {
            let password = "пароль密码🔐";
            let result = UserService::hash_password(password);
            assert!(result.is_ok());
        }
    }

    mod integration_tests {
        use super::*;
        use crate::test_utils::create_test_pool;
        use crate::dto::{CreateUserDto, PageRequest, UpdateUserDto};
        // Phase 2a (2026-05-11): parametrized cases for the 8-way sort match
        // in find_all. rstest's `#[case]` annotations let one test function
        // cover 17 sort permutations (8 fields × 2 directions + 1 default-
        // unknown) without duplicating fixture setup. Locked decision #9.
        use rstest::rstest;

        #[test]
        fn test_create_user() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            let dto = CreateUserDto {
                login: "testuser".to_string(),
                password: Some("password123".to_string()),
                first_name: Some("Test".to_string()),
                last_name: Some("User".to_string()),
                email: "testuser@example.com".to_string(),
                activated: Some(true),
                lang_key: Some("en".to_string()),
                image_url: None,
                authorities: Some(vec!["ROLE_USER".to_string()]),
            };

            let result = UserService::create(&mut conn, dto, "system");
            assert!(result.is_ok());

            let user = result.unwrap();
            assert_eq!(user.login, "testuser");
            assert_eq!(user.email, "testuser@example.com");
            assert!(user.activated);
        }

        #[test]
        fn test_create_user_duplicate_login() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            let dto = CreateUserDto {
                login: "duplicate".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "first@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: None,
            };

            // First creation should succeed
            assert!(UserService::create(&mut conn, dto.clone(), "system").is_ok());

            // Second creation with same login should fail
            let dto2 = CreateUserDto {
                login: "duplicate".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "second@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: None,
            };

            let result = UserService::create(&mut conn, dto2, "system");
            assert!(result.is_err());
        }

        #[test]
        fn test_create_user_duplicate_email() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            let dto = CreateUserDto {
                login: "user1".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "same@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: None,
            };

            // First creation should succeed
            assert!(UserService::create(&mut conn, dto, "system").is_ok());

            // Second creation with same email should fail
            let dto2 = CreateUserDto {
                login: "user2".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "same@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: None,
            };

            let result = UserService::create(&mut conn, dto2, "system");
            assert!(result.is_err());
        }

        #[test]
        fn test_find_by_id() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            // Create a user first
            let dto = CreateUserDto {
                login: "findbyid".to_string(),
                password: Some("password123".to_string()),
                first_name: Some("Find".to_string()),
                last_name: Some("ById".to_string()),
                email: "findbyid@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: None,
            };

            let created = UserService::create(&mut conn, dto, "system").unwrap();
            let found = UserService::find_by_id(&mut conn, created.id).unwrap();

            assert_eq!(found.login, "findbyid");
            assert_eq!(found.email, "findbyid@example.com");
        }

        #[test]
        fn test_find_by_id_not_found() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            let result = UserService::find_by_id(&mut conn, 99999);
            assert!(result.is_err());
        }

        #[test]
        fn test_find_by_login() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            let dto = CreateUserDto {
                login: "findbylogin".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "findbylogin@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: None,
            };

            UserService::create(&mut conn, dto, "system").unwrap();
            let found = UserService::find_by_login(&mut conn, "findbylogin").unwrap();

            assert_eq!(found.login, "findbylogin");
        }

        #[test]
        fn test_find_by_email() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            let dto = CreateUserDto {
                login: "findbyemail".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "findbyemail@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: None,
            };

            UserService::create(&mut conn, dto, "system").unwrap();
            let found = UserService::find_by_email(&mut conn, "findbyemail@example.com").unwrap();

            assert_eq!(found.login, "findbyemail");
        }

        #[test]
        fn test_find_all_with_pagination() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            // Count existing users from migrations (admin and user are seeded)
            let initial_page_request = PageRequest {
                page: Some(0),
                size: Some(100),
                sort: Vec::new(),
            };
            let (_, initial_count) = UserService::find_all(&mut conn, &initial_page_request).unwrap();

            // Create multiple users
            for i in 0..5 {
                let dto = CreateUserDto {
                    login: format!("paguser{}", i),
                    password: Some("password123".to_string()),
                    first_name: Some(format!("User{}", i)),
                    last_name: None,
                    email: format!("paguser{}@example.com", i),
                    activated: Some(true),
                    lang_key: None,
                    image_url: None,
                    authorities: None,
                };
                UserService::create(&mut conn, dto, "system").unwrap();
            }

            // Test pagination
            let page_request = PageRequest {
                page: Some(0),
                size: Some(2),
                sort: Vec::new(),
            };

            let (users, total) = UserService::find_all(&mut conn, &page_request).unwrap();
            assert_eq!(users.len(), 2);
            assert_eq!(total, initial_count + 5); // 5 new users plus any seeded users
        }

        #[test]
        fn test_find_all_with_sorting() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            // Create users with different logins that sort after 'admin' and 'user' (the seeded users)
            for login in ["zcharlie", "zalice", "zbob"] {
                let dto = CreateUserDto {
                    login: login.to_string(),
                    password: Some("password123".to_string()),
                    first_name: None,
                    last_name: None,
                    email: format!("{}@example.com", login),
                    activated: Some(true),
                    lang_key: None,
                    image_url: None,
                    authorities: None,
                };
                UserService::create(&mut conn, dto, "system").unwrap();
            }

            // Test ascending sort - admin comes first, then user, then z* users
            let page_request = PageRequest {
                page: Some(0),
                size: Some(10),
                sort: vec!["login,asc".to_string()],
            };

            let (users, _) = UserService::find_all(&mut conn, &page_request).unwrap();
            // First two are admin and user from migrations
            // Then z* users in alphabetical order
            let z_users: Vec<_> = users.iter().filter(|u| u.login.starts_with('z')).collect();
            assert!(z_users.len() >= 3);
            assert_eq!(z_users[0].login, "zalice");
            assert_eq!(z_users[1].login, "zbob");
            assert_eq!(z_users[2].login, "zcharlie");
        }

        /// Insert 3 users with distinct values for every string-sortable field.
        /// Login suffix matches the alphabetical position so callers can spot-
        /// check ordering by looking at the trailing letter.
        ///
        /// Insertion order is `b, a, c` — deliberately scrambled so a test
        /// that mistakenly asserts insertion order rather than sort order
        /// would fail loudly. Returns the prefix string for filtering.
        fn setup_three_sortable_users(conn: &mut DbConnection, prefix: &str) {
            let users = [
                // (login_suffix, email_prefix, first_name, last_name, lang_key)
                ("b", "beta",  "Beta",  "Lname1", "fr"),
                ("a", "alpha", "Alpha", "Lname2", "en"),
                ("c", "gamma", "Gamma", "Lname3", "es"),
            ];
            for (suffix, email_p, fn_, ln, lk) in users {
                let dto = CreateUserDto {
                    login: format!("{}_{}", prefix, suffix),
                    password: Some("password123".to_string()),
                    first_name: Some(fn_.to_string()),
                    last_name: Some(ln.to_string()),
                    email: format!("{}_{}@example.com", email_p, prefix),
                    activated: Some(true),
                    lang_key: Some(lk.to_string()),
                    image_url: None,
                    authorities: None,
                };
                UserService::create(conn, dto, "system").unwrap();
            }
        }

        /// Filter find_all results to only the users this test created, by
        /// matching on the unique login prefix. Returns just the suffix
        /// letters in result order — the value tests assert against.
        fn extract_suffixes(users: &[User], prefix: &str) -> Vec<String> {
            users
                .iter()
                .filter_map(|u| u.login.strip_prefix(&format!("{}_", prefix)).map(String::from))
                .collect()
        }

        // Parametrized sort coverage for the 5 string-sortable fields plus
        // the default-unknown-field fall-through. Boolean and date fields
        // need different fixtures (see the next two test functions).
        //
        // Each case lists the expected login suffix order. Suffixes are a/b/c;
        // setup inserts in order b, a, c (scrambled to catch insertion-order
        // assertion bugs). The expected vec tells you what the sort should
        // produce regardless of insertion order.
        #[rstest]
        #[case::login_asc("login", "asc", "slo", &["a", "b", "c"])]
        #[case::login_desc("login", "desc", "sld", &["c", "b", "a"])]
        #[case::email_asc("email", "asc", "sea", &["a", "b", "c"])]
        #[case::email_desc("email", "desc", "sed", &["c", "b", "a"])]
        #[case::first_name_asc("firstName", "asc", "sfa", &["a", "b", "c"])]
        #[case::first_name_desc("firstName", "desc", "sfd", &["c", "b", "a"])]
        #[case::last_name_asc("lastName", "asc", "sla", &["b", "a", "c"])]
        #[case::last_name_desc("lastName", "desc", "sln", &["c", "a", "b"])]
        #[case::lang_key_asc("langKey", "asc", "ska", &["a", "c", "b"])]
        #[case::lang_key_desc("langKey", "desc", "skd", &["b", "c", "a"])]
        // Default arm: unknown field falls back to id ASC, which in postgres
        // matches insertion order. Setup inserted b, a, c — so id asc = b, a, c.
        #[case::unknown_field_falls_back_to_id_asc("not_a_field", "asc", "sun", &["b", "a", "c"])]
        fn test_find_all_sorts_by_string_field(
            #[case] field: &str,
            #[case] direction: &str,
            #[case] prefix: &str,
            #[case] expected_suffixes: &[&str],
        ) {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();
            setup_three_sortable_users(&mut conn, prefix);

            // size=100 ensures all test users fit on page 0 alongside seeded admin/user.
            let page_request = PageRequest {
                page: Some(0),
                size: Some(100),
                sort: vec![format!("{},{}", field, direction)],
            };
            let (users, _total) = UserService::find_all(&mut conn, &page_request).unwrap();
            let actual = extract_suffixes(&users, prefix);
            let expected: Vec<String> = expected_suffixes.iter().map(|s| s.to_string()).collect();
            assert_eq!(
                actual, expected,
                "sort={},{} expected suffixes {:?} got {:?}",
                field, direction, expected, actual
            );
        }

        // Boolean field needs a 2-state fixture: one user activated=false,
        // others activated=true. Within the activated=true group, secondary
        // ordering is undefined, so the assertion only pins where the
        // activated=false user sits relative to the activated=true ones.
        #[rstest]
        #[case::activated_asc("asc", "sba", false)]   // false (0) sorts first
        #[case::activated_desc("desc", "sbd", true)]  // true (1) sorts first
        fn test_find_all_sorts_by_activated(
            #[case] direction: &str,
            #[case] prefix: &str,
            #[case] first_user_activated: bool,
        ) {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();
            // Two users: one activated, one not.
            for (suffix, activated) in [("on", true), ("off", false)] {
                let dto = CreateUserDto {
                    login: format!("{}_{}", prefix, suffix),
                    password: Some("password123".to_string()),
                    first_name: None,
                    last_name: None,
                    email: format!("{}_{}@x.com", prefix, suffix),
                    activated: Some(activated),
                    lang_key: None,
                    image_url: None,
                    authorities: None,
                };
                UserService::create(&mut conn, dto, "system").unwrap();
            }
            let page_request = PageRequest {
                page: Some(0),
                size: Some(100),
                sort: vec![format!("activated,{}", direction)],
            };
            let (users, _) = UserService::find_all(&mut conn, &page_request).unwrap();
            // Filter to our test users and find the first one in result order.
            let ours: Vec<&User> = users
                .iter()
                .filter(|u| u.login.starts_with(&format!("{}_", prefix)))
                .collect();
            assert_eq!(ours.len(), 2, "expected 2 test users, got {}", ours.len());
            assert_eq!(
                ours[0].activated, first_user_activated,
                "sort=activated,{} expected first user activated={} got activated={}",
                direction, first_user_activated, ours[0].activated
            );
        }

        // Date-field sort: insertion order produces increasing created_date /
        // last_modified_date (postgres NOW() at insert time). Sleeps between
        // inserts guarantee distinct microsecond-precision timestamps so the
        // sort order is deterministic. `last_modified_date` equals
        // `created_date` for never-updated users, so the assertion shape
        // matches across both fields.
        #[rstest]
        #[case::created_date_asc("createdDate", "asc", "sca", &["a", "b", "c"])]
        #[case::created_date_desc("createdDate", "desc", "scd", &["c", "b", "a"])]
        #[case::last_modified_date_asc("lastModifiedDate", "asc", "sma", &["a", "b", "c"])]
        #[case::last_modified_date_desc("lastModifiedDate", "desc", "smd", &["c", "b", "a"])]
        fn test_find_all_sorts_by_date_field(
            #[case] field: &str,
            #[case] direction: &str,
            #[case] prefix: &str,
            #[case] expected_suffixes: &[&str],
        ) {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();
            // Insert in alphabetical order with sleeps so a < b < c in time.
            for suffix in ["a", "b", "c"] {
                let dto = CreateUserDto {
                    login: format!("{}_{}", prefix, suffix),
                    password: Some("password123".to_string()),
                    first_name: None,
                    last_name: None,
                    email: format!("{}_{}@x.com", prefix, suffix),
                    activated: Some(true),
                    lang_key: None,
                    image_url: None,
                    authorities: None,
                };
                UserService::create(&mut conn, dto, "system").unwrap();
                // 10ms separation is more than enough for postgres timestamp
                // resolution on any sane runner.
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let page_request = PageRequest {
                page: Some(0),
                size: Some(100),
                sort: vec![format!("{},{}", field, direction)],
            };
            let (users, _) = UserService::find_all(&mut conn, &page_request).unwrap();
            let actual = extract_suffixes(&users, prefix);
            let expected: Vec<String> = expected_suffixes.iter().map(|s| s.to_string()).collect();
            assert_eq!(
                actual, expected,
                "sort={},{} expected {:?} got {:?}",
                field, direction, expected, actual
            );
        }

        #[test]
        fn test_update_user() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            // Create a user
            let dto = CreateUserDto {
                login: "toupdate".to_string(),
                password: Some("password123".to_string()),
                first_name: Some("Original".to_string()),
                last_name: Some("Name".to_string()),
                email: "toupdate@example.com".to_string(),
                activated: Some(false),
                lang_key: None,
                image_url: None,
                authorities: None,
            };

            let created = UserService::create(&mut conn, dto, "system").unwrap();

            // Update the user
            let update_dto = UpdateUserDto {
                login: None,
                first_name: Some("Updated".to_string()),
                last_name: Some("User".to_string()),
                email: Some("updated@example.com".to_string()),
                activated: Some(true),
                lang_key: Some("fr".to_string()),
                image_url: None,
                authorities: None,
            };

            let updated = UserService::update(&mut conn, created.id, update_dto, "admin").unwrap();

            assert_eq!(updated.first_name, Some("Updated".to_string()));
            assert_eq!(updated.last_name, Some("User".to_string()));
            assert_eq!(updated.email, "updated@example.com");
            assert!(updated.activated);
            assert_eq!(updated.lang_key, Some("fr".to_string()));
        }

        #[test]
        fn test_update_user_authorities() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            // Create a user with USER role
            let dto = CreateUserDto {
                login: "authupdate".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "authupdate@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: Some(vec!["ROLE_USER".to_string()]),
            };

            let created = UserService::create(&mut conn, dto, "system").unwrap();

            // Update to add ADMIN role
            let update_dto = UpdateUserDto {
                login: None,
                first_name: None,
                last_name: None,
                email: None,
                activated: None,
                lang_key: None,
                image_url: None,
                authorities: Some(vec!["ROLE_USER".to_string(), "ROLE_ADMIN".to_string()]),
            };

            UserService::update(&mut conn, created.id, update_dto, "admin").unwrap();

            let authorities = UserService::get_authorities(&mut conn, created.id).unwrap();
            assert!(authorities.contains(&"ROLE_USER".to_string()));
            assert!(authorities.contains(&"ROLE_ADMIN".to_string()));
        }

        #[test]
        fn test_delete_user() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            // Create a user
            let dto = CreateUserDto {
                login: "todelete".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "todelete@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: None,
            };

            let created = UserService::create(&mut conn, dto, "system").unwrap();

            // Delete the user
            let result = UserService::delete(&mut conn, created.id);
            assert!(result.is_ok());

            // Verify user is deleted
            let find_result = UserService::find_by_id(&mut conn, created.id);
            assert!(find_result.is_err());
        }

        #[test]
        fn test_get_authorities() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            let dto = CreateUserDto {
                login: "withauth".to_string(),
                password: Some("password123".to_string()),
                first_name: None,
                last_name: None,
                email: "withauth@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: Some(vec!["ROLE_USER".to_string(), "ROLE_ADMIN".to_string()]),
            };

            let created = UserService::create(&mut conn, dto, "system").unwrap();
            let authorities = UserService::get_authorities(&mut conn, created.id).unwrap();

            assert_eq!(authorities.len(), 2);
            assert!(authorities.contains(&"ROLE_USER".to_string()));
            assert!(authorities.contains(&"ROLE_ADMIN".to_string()));
        }

        #[test]
        fn test_update_password() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            let dto = CreateUserDto {
                login: "pwdupdate".to_string(),
                password: Some("oldpassword".to_string()),
                first_name: None,
                last_name: None,
                email: "pwdupdate@example.com".to_string(),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: None,
            };

            UserService::create(&mut conn, dto, "system").unwrap();

            // Update password
            let new_hash = UserService::hash_password("newpassword").unwrap();
            let result = UserService::update_password(&mut conn, "pwdupdate", &new_hash);
            assert!(result.is_ok());

            // Verify password was updated
            let user = UserService::find_by_login(&mut conn, "pwdupdate").unwrap();
            assert_eq!(user.password_hash, new_hash);
        }

        #[test]
        fn test_create_with_authorities() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();

            let password_hash = UserService::hash_password("password123").unwrap();
            let new_user = NewUser {
                login: "directcreate".to_string(),
                password_hash,
                first_name: Some("Direct".to_string()),
                last_name: Some("Create".to_string()),
                email: "directcreate@example.com".to_string(),
                activated: true,
                onboarding_completed: false,
                lang_key: Some("en".to_string()),
                image_url: None,
                created_by: Some("test".to_string()),
                created_date: Some(chrono::Utc::now().naive_utc()),
                last_modified_by: Some("test".to_string()),
                last_modified_date: Some(chrono::Utc::now().naive_utc()),
            };

            let result = UserService::create_with_authorities(
                &mut conn,
                new_user,
                vec!["ROLE_USER".to_string(), "ROLE_ADMIN".to_string()],
            );

            assert!(result.is_ok());
            let user = result.unwrap();
            assert_eq!(user.login, "directcreate");

            let authorities = UserService::get_authorities(&mut conn, user.id).unwrap();
            assert_eq!(authorities.len(), 2);
        }

        // Phase 2a step 3 (2026-05-11): error-path and idempotent-contract
        // tests. Each test pins the specific contract — `assert!(matches!(...))`
        // on the AppError variant for real error paths; `Ok(())` plus a
        // post-condition for idempotent operations (delete and update_password
        // are intentionally not-an-error when the target row doesn't exist).
        // The contracts here are easy to mis-remember; pinning them in tests
        // is the cheapest way to keep future refactors honest.

        #[test]
        fn test_find_by_login_returns_not_found_for_unknown_login() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();
            let result = UserService::find_by_login(&mut conn, "definitely_not_a_real_user_xyz");
            match result {
                Err(AppError::NotFound(msg)) => {
                    assert!(msg.contains("definitely_not_a_real_user_xyz"), "got: {}", msg);
                }
                other => panic!("Expected NotFound, got {:?}", other),
            }
        }

        #[test]
        fn test_find_by_email_returns_not_found_for_unknown_email() {
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();
            let result = UserService::find_by_email(&mut conn, "nobody@nowhere.invalid");
            assert!(matches!(result, Err(AppError::NotFound(_))));
        }

        #[test]
        fn test_update_returns_not_found_for_nonexistent_id() {
            // The diesel UPDATE silently affects zero rows when the id doesn't
            // match (postgres `UPDATE ... WHERE id = X` with no match). The
            // NotFound surfaces from the trailing `find_by_id` that the update
            // method calls to return the updated row.
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();
            let dto = UpdateUserDto {
                login: None,
                first_name: Some("Doesnt".to_string()),
                last_name: Some("Matter".to_string()),
                email: Some("doesnt@matter.com".to_string()),
                activated: Some(true),
                lang_key: None,
                image_url: None,
                authorities: None,
            };
            let result = UserService::update(&mut conn, i32::MAX, dto, "system");
            match result {
                Err(AppError::NotFound(msg)) => assert!(msg.contains(&i32::MAX.to_string()), "got: {}", msg),
                other => panic!("Expected NotFound, got {:?}", other),
            }
        }

        #[test]
        fn test_delete_is_idempotent_for_nonexistent_id() {
            // Contract: delete returns Ok(()) when the id doesn't exist.
            // This is intentional — DELETE on zero rows is not an error in
            // SQL, and the service surface preserves that. Pinning the
            // contract so a future "make delete strict" refactor produces
            // an explicit test failure rather than a silent behavior change.
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();
            let result = UserService::delete(&mut conn, i32::MAX);
            assert!(result.is_ok(), "delete of nonexistent id should succeed, got {:?}", result);
        }

        #[test]
        fn test_update_password_is_idempotent_for_nonexistent_login() {
            // Same idempotent contract as delete: UPDATE matching zero rows
            // returns Ok(()). Pin the contract to catch silent regressions.
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();
            let fake_hash = "$argon2id$placeholder";
            let result = UserService::update_password(&mut conn, "no_such_login_xyz", fake_hash);
            assert!(result.is_ok(), "update_password on nonexistent login should succeed, got {:?}", result);
        }

        #[test]
        fn test_get_authorities_returns_empty_for_nonexistent_user() {
            // Documents the contract: querying authorities for an unknown
            // user_id returns Ok(empty_vec), NOT NotFound. The join semantics
            // (LEFT JOIN-style filter) naturally yield zero rows.
            let pool = create_test_pool();
            let mut conn = pool.get().unwrap();
            let result = UserService::get_authorities(&mut conn, i32::MAX);
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }
    }
}
