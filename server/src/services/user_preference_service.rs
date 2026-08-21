use chrono::Utc;
use diesel::prelude::*;

use crate::db::connection::DbConnection;
use crate::db::schema::user_preference;
use crate::dto::{CreateUserPreferenceDto, PageRequest, UpdateUserPreferenceDto};
use crate::errors::AppError;
use crate::models::{NewUserPreference, UpdateUserPreference, UserPreference};

pub struct UserPreferenceService;

impl UserPreferenceService {
    /// Find all userPreferences with pagination
    pub fn find_all(
        conn: &mut DbConnection,
        page_request: &PageRequest,
    ) -> Result<(Vec<UserPreference>, i64), AppError> {
        let page = page_request.page.unwrap_or(0);
        let size = page_request.size.unwrap_or(20).min(100);
        let offset = page * size;

        let total: i64 = user_preference::table
            .count()
            .get_result(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Get primary sort parameter (format: "field,direction" e.g., "name,asc" or "cost,desc")
        let (sort_field, sort_dir) = page_request.primary_sort().unwrap_or(("id", "asc"));

        let is_desc = sort_dir.eq_ignore_ascii_case("desc");

        // Dynamic sorting based on field name
        let results = match sort_field {
            "userId" | "user_id" => {
                if is_desc {
                    user_preference::table
                        .order(user_preference::user_id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::user_id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
            "remoteOnly" | "remote_only" => {
                if is_desc {
                    user_preference::table
                        .order(user_preference::remote_only.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::remote_only.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
            "contractType" | "contract_type" => {
                if is_desc {
                    user_preference::table
                        .order(user_preference::contract_type.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::contract_type.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
            "salaryMin" | "salary_min" => {
                if is_desc {
                    user_preference::table
                        .order(user_preference::salary_min.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::salary_min.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
            "salaryMax" | "salary_max" => {
                if is_desc {
                    user_preference::table
                        .order(user_preference::salary_max.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::salary_max.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
            "preferredRoles" | "preferred_roles" => {
                if is_desc {
                    user_preference::table
                        .order(user_preference::preferred_roles.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::preferred_roles.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
            "excludedTechnologies" | "excluded_technologies" => {
                if is_desc {
                    user_preference::table
                        .order(user_preference::excluded_technologies.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::excluded_technologies.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
            "preferredLocations" | "preferred_locations" => {
                if is_desc {
                    user_preference::table
                        .order(user_preference::preferred_locations.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::preferred_locations.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
            "createdDate" | "created_date" => {
                if is_desc {
                    user_preference::table
                        .order(user_preference::created_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::created_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
            "lastModifiedDate" | "last_modified_date" => {
                if is_desc {
                    user_preference::table
                        .order(user_preference::last_modified_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::last_modified_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
            _ => {
                // Default: sort by id
                if is_desc {
                    user_preference::table
                        .order(user_preference::id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                } else {
                    user_preference::table
                        .order(user_preference::id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(UserPreference::as_select()).load(conn)
                }
            }
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((results, total))
    }

    /// Find userPreference by ID
    pub fn find_by_id(conn: &mut DbConnection, id: i32) -> Result<UserPreference, AppError> {
        user_preference::table
            .find(id)
            .select(UserPreference::as_select()).first(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound(format!("UserPreference {} not found", id))
                }
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Create a new userPreference
    pub fn create(
        conn: &mut DbConnection,
        dto: CreateUserPreferenceDto,
        created_by: &str,
    ) -> Result<UserPreference, AppError> {
        let now = Utc::now().naive_utc();

        let new_entity = NewUserPreference {
            user_id: dto.user_id,
            remote_only: dto.remote_only,
            contract_type: dto.contract_type,
            salary_min: dto.salary_min,
            salary_max: dto.salary_max,
            preferred_roles: dto.preferred_roles,
            excluded_technologies: dto.excluded_technologies,
            preferred_locations: dto.preferred_locations,
            created_by: Some(created_by.to_string()),
            created_date: Some(now),
            last_modified_by: Some(created_by.to_string()),
            last_modified_date: Some(now),
        };

        let entity = diesel::insert_into(user_preference::table)
            .values(&new_entity)
            .get_result::<UserPreference>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(entity)
    }

    /// Update an existing userPreference
    pub fn update(
        conn: &mut DbConnection,
        id: i32,
        dto: UpdateUserPreferenceDto,
        modified_by: &str,
    ) -> Result<UserPreference, AppError> {
        let now = Utc::now().naive_utc();

        let update = UpdateUserPreference {
            user_id: dto.user_id,
            remote_only: dto.remote_only,
            contract_type: dto.contract_type,
            salary_min: dto.salary_min,
            salary_max: dto.salary_max,
            preferred_roles: dto.preferred_roles,
            excluded_technologies: dto.excluded_technologies,
            preferred_locations: dto.preferred_locations,
            last_modified_by: Some(modified_by.to_string()),
            last_modified_date: Some(now),
        };

        diesel::update(user_preference::table.find(id))
            .set(&update)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Self::find_by_id(conn, id)
    }

    /// Delete a userPreference
    pub fn delete(conn: &mut DbConnection, id: i32) -> Result<(), AppError> {
        diesel::delete(user_preference::table.find(id))
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_pool;
    use crate::dto::PageRequest;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn create_test_dto_with_suffix(suffix: u32) -> CreateUserPreferenceDto {
        let _ = suffix; // Suppress unused warning when there are no String fields
        CreateUserPreferenceDto {
            user_id: format!("test_value_{}", suffix),
            remote_only: None,
            contract_type: None,
            salary_min: None,
            salary_max: None,
            preferred_roles: None,
            excluded_technologies: None,
            preferred_locations: None,
        }
    }

    fn create_test_dto() -> CreateUserPreferenceDto {
        let suffix = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        create_test_dto_with_suffix(suffix)
    }

    #[test]
    fn test_create_userPreference() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let dto = create_test_dto();
        let result = UserPreferenceService::create(&mut conn, dto, "test_user");

        assert!(result.is_ok());
        let entity = result.unwrap();
        assert!(entity.id > 0);
    }

    #[test]
    fn test_find_by_id() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity first
        let dto = create_test_dto();
        let created = UserPreferenceService::create(&mut conn, dto, "test_user").unwrap();

        // Find it by ID
        let result = UserPreferenceService::find_by_id(&mut conn, created.id);
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let result = UserPreferenceService::find_by_id(&mut conn, 99999);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_all() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create a few entities with unique values
        for _ in 0..3 {
            let dto = create_test_dto();
            UserPreferenceService::create(&mut conn, dto, "test_user").unwrap();
        }

        let page_request = PageRequest {
            page: Some(0),
            size: Some(10),
            sort: Vec::new(),
        };

        let result = UserPreferenceService::find_all(&mut conn, &page_request);
        assert!(result.is_ok());
        let (items, total) = result.unwrap();
        assert!(items.len() >= 3);
        assert!(total >= 3);
    }

    #[test]
    fn test_find_all_with_pagination() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create 5 entities with unique values
        for _ in 0..5 {
            let dto = create_test_dto();
            UserPreferenceService::create(&mut conn, dto, "test_user").unwrap();
        }

        // Request page 0 with size 2
        let page_request = PageRequest {
            page: Some(0),
            size: Some(2),
            sort: Vec::new(),
        };

        let result = UserPreferenceService::find_all(&mut conn, &page_request);
        assert!(result.is_ok());
        let (items, _total) = result.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_update_userPreference() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = UserPreferenceService::create(&mut conn, dto, "test_user").unwrap();

        // Update it
        let update_dto = UpdateUserPreferenceDto {
            user_id: Some("updated_value".to_string()),
            remote_only: Some(true),
            contract_type: Some("updated_value".to_string()),
            salary_min: Some(999),
            salary_max: Some(999),
            preferred_roles: Some("updated_value".to_string()),
            excluded_technologies: Some("updated_value".to_string()),
            preferred_locations: Some("updated_value".to_string()),
        };

        let result = UserPreferenceService::update(&mut conn, created.id, update_dto, "test_user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_userPreference() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = UserPreferenceService::create(&mut conn, dto, "test_user").unwrap();

        // Delete it
        let result = UserPreferenceService::delete(&mut conn, created.id);
        assert!(result.is_ok());

        // Verify it's gone
        let find_result = UserPreferenceService::find_by_id(&mut conn, created.id);
        assert!(find_result.is_err());
    }

    // Phase 2d (v1.0.1, 2026-06-07): exercise every sort match arm.
    //
    // The existing test_find_all uses `sort: Vec::new()` which dispatches
    // to the `_ => id ASC` default branch — every field-specific arm of
    // the match in find_all sits uncovered. This test walks the dispatch
    // table emitted by the entity template for this entity's specific
    // field list, asserting each (field × direction) sort returns Ok.
    //
    // Ordering isn't asserted per-arm because the underlying diesel
    // .order() semantics are pinned by user_service Phase 2a's
    // parametrized tests; the value here is template-coverage —
    // catching "generator emitted wrong column in this arm" regressions.
    // Lives in the entity service template so coverage flows to every
    // scaffold with entities (Product, Category in microservice-cb; any
    // future entity inherits the same coverage shape).
    #[test]
    fn test_find_all_exercises_all_sort_arms() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Three entities ensures every sort returns a non-empty page;
        // also pins that the default page-size cap (100) isn't a factor.
        for _ in 0..3 {
            let dto = create_test_dto();
            UserPreferenceService::create(&mut conn, dto, "test_user").unwrap();
        }

        let sort_specs: &[&str] = &[
            "userId,asc",
            "userId,desc",
            "remoteOnly,asc",
            "remoteOnly,desc",
            "contractType,asc",
            "contractType,desc",
            "salaryMin,asc",
            "salaryMin,desc",
            "salaryMax,asc",
            "salaryMax,desc",
            "preferredRoles,asc",
            "preferredRoles,desc",
            "excludedTechnologies,asc",
            "excludedTechnologies,desc",
            "preferredLocations,asc",
            "preferredLocations,desc",
            "createdDate,asc",
            "createdDate,desc",
            "lastModifiedDate,asc",
            "lastModifiedDate,desc",
            // Default-unknown-field branch (asc and desc both fall through
            // to the `_ => id ASC/DESC` arm, exercising both is_desc paths).
            "not_a_field,asc",
            "not_a_field,desc",
        ];

        for spec in sort_specs {
            let page_request = PageRequest {
                page: Some(0),
                size: Some(100),
                sort: vec![spec.to_string()],
            };
            let result = UserPreferenceService::find_all(&mut conn, &page_request);
            assert!(
                result.is_ok(),
                "sort spec {:?} failed: {:?}",
                spec,
                result.as_ref().err()
            );
            let (items, _total) = result.unwrap();
            assert!(
                items.len() >= 3,
                "sort spec {:?} returned only {} items; the 3 created above should appear on page 0",
                spec,
                items.len()
            );
        }
    }
}
