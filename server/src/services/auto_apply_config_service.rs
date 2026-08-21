use chrono::Utc;
use diesel::prelude::*;

use crate::db::connection::DbConnection;
use crate::db::schema::auto_apply_config;
use crate::dto::{CreateAutoApplyConfigDto, PageRequest, UpdateAutoApplyConfigDto};
use crate::errors::AppError;
use crate::models::{NewAutoApplyConfig, UpdateAutoApplyConfig, AutoApplyConfig};
use uuid::Uuid;

pub struct AutoApplyConfigService;

impl AutoApplyConfigService {
    /// Find all autoApplyConfigs with pagination
    pub fn find_all(
        conn: &mut DbConnection,
        page_request: &PageRequest,
    ) -> Result<(Vec<AutoApplyConfig>, i64), AppError> {
        let page = page_request.page.unwrap_or(0);
        let size = page_request.size.unwrap_or(20).min(100);
        let offset = page * size;

        let total: i64 = auto_apply_config::table
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
                    auto_apply_config::table
                        .order(auto_apply_config::user_id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                } else {
                    auto_apply_config::table
                        .order(auto_apply_config::user_id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                }
            }
            "mode" => {
                if is_desc {
                    auto_apply_config::table
                        .order(auto_apply_config::mode.desc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                } else {
                    auto_apply_config::table
                        .order(auto_apply_config::mode.asc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                }
            }
            "minScore" | "min_score" => {
                if is_desc {
                    auto_apply_config::table
                        .order(auto_apply_config::min_score.desc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                } else {
                    auto_apply_config::table
                        .order(auto_apply_config::min_score.asc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                }
            }
            "maxPerDay" | "max_per_day" => {
                if is_desc {
                    auto_apply_config::table
                        .order(auto_apply_config::max_per_day.desc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                } else {
                    auto_apply_config::table
                        .order(auto_apply_config::max_per_day.asc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                }
            }
            "sources" => {
                if is_desc {
                    auto_apply_config::table
                        .order(auto_apply_config::sources.desc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                } else {
                    auto_apply_config::table
                        .order(auto_apply_config::sources.asc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                }
            }
            "createdDate" | "created_date" => {
                if is_desc {
                    auto_apply_config::table
                        .order(auto_apply_config::created_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                } else {
                    auto_apply_config::table
                        .order(auto_apply_config::created_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                }
            }
            "lastModifiedDate" | "last_modified_date" => {
                if is_desc {
                    auto_apply_config::table
                        .order(auto_apply_config::last_modified_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                } else {
                    auto_apply_config::table
                        .order(auto_apply_config::last_modified_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                }
            }
            _ => {
                // Default: sort by id
                if is_desc {
                    auto_apply_config::table
                        .order(auto_apply_config::id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                } else {
                    auto_apply_config::table
                        .order(auto_apply_config::id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(AutoApplyConfig::as_select()).load(conn)
                }
            }
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((results, total))
    }

    /// Find autoApplyConfig by ID
    pub fn find_by_id(conn: &mut DbConnection, id: Uuid) -> Result<AutoApplyConfig, AppError> {
        auto_apply_config::table
            .find(id)
            .select(AutoApplyConfig::as_select()).first(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound(format!("AutoApplyConfig {} not found", id))
                }
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Create a new autoApplyConfig
    pub fn create(
        conn: &mut DbConnection,
        dto: CreateAutoApplyConfigDto,
        created_by: &str,
    ) -> Result<AutoApplyConfig, AppError> {
        let now = Utc::now().naive_utc();

        let new_entity = NewAutoApplyConfig {
            user_id: dto.user_id,
            mode: dto.mode,
            min_score: dto.min_score,
            max_per_day: dto.max_per_day,
            sources: dto.sources,
            created_by: Some(created_by.to_string()),
            created_date: Some(now),
            last_modified_by: Some(created_by.to_string()),
            last_modified_date: Some(now),
        };

        let entity = diesel::insert_into(auto_apply_config::table)
            .values(&new_entity)
            .get_result::<AutoApplyConfig>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(entity)
    }

    /// Update an existing autoApplyConfig
    pub fn update(
        conn: &mut DbConnection,
        id: Uuid,
        dto: UpdateAutoApplyConfigDto,
        modified_by: &str,
    ) -> Result<AutoApplyConfig, AppError> {
        let now = Utc::now().naive_utc();

        let update = UpdateAutoApplyConfig {
            user_id: dto.user_id,
            mode: dto.mode,
            min_score: dto.min_score,
            max_per_day: dto.max_per_day,
            sources: dto.sources,
            last_modified_by: Some(modified_by.to_string()),
            last_modified_date: Some(now),
        };

        diesel::update(auto_apply_config::table.find(id))
            .set(&update)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Self::find_by_id(conn, id)
    }

    /// Delete a autoApplyConfig
    pub fn delete(conn: &mut DbConnection, id: Uuid) -> Result<(), AppError> {
        diesel::delete(auto_apply_config::table.find(id))
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

    fn create_test_dto_with_suffix(suffix: u32) -> CreateAutoApplyConfigDto {
        let _ = suffix; // Suppress unused warning when there are no String fields
        CreateAutoApplyConfigDto {
            user_id: format!("test_value_{}", suffix),
            mode: None,
            min_score: None,
            max_per_day: None,
            sources: None,
        }
    }

    fn create_test_dto() -> CreateAutoApplyConfigDto {
        let suffix = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        create_test_dto_with_suffix(suffix)
    }

    #[test]
    fn test_create_autoApplyConfig() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let dto = create_test_dto();
        let result = AutoApplyConfigService::create(&mut conn, dto, "test_user");

        assert!(result.is_ok());
        let entity = result.unwrap();
        assert!(!entity.id.is_nil());
    }

    #[test]
    fn test_find_by_id() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity first
        let dto = create_test_dto();
        let created = AutoApplyConfigService::create(&mut conn, dto, "test_user").unwrap();

        // Find it by ID
        let result = AutoApplyConfigService::find_by_id(&mut conn, created.id);
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let result = AutoApplyConfigService::find_by_id(&mut conn, Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn test_find_all() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create a few entities with unique values
        for _ in 0..3 {
            let dto = create_test_dto();
            AutoApplyConfigService::create(&mut conn, dto, "test_user").unwrap();
        }

        let page_request = PageRequest {
            page: Some(0),
            size: Some(10),
            sort: Vec::new(),
        };

        let result = AutoApplyConfigService::find_all(&mut conn, &page_request);
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
            AutoApplyConfigService::create(&mut conn, dto, "test_user").unwrap();
        }

        // Request page 0 with size 2
        let page_request = PageRequest {
            page: Some(0),
            size: Some(2),
            sort: Vec::new(),
        };

        let result = AutoApplyConfigService::find_all(&mut conn, &page_request);
        assert!(result.is_ok());
        let (items, _total) = result.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_update_autoApplyConfig() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = AutoApplyConfigService::create(&mut conn, dto, "test_user").unwrap();

        // Update it
        let update_dto = UpdateAutoApplyConfigDto {
            user_id: Some("updated_value".to_string()),
            mode: Some("updated_value".to_string()),
            min_score: Some(999.99),
            max_per_day: Some(999),
            sources: Some("updated_value".to_string()),
        };

        let result = AutoApplyConfigService::update(&mut conn, created.id, update_dto, "test_user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_autoApplyConfig() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = AutoApplyConfigService::create(&mut conn, dto, "test_user").unwrap();

        // Delete it
        let result = AutoApplyConfigService::delete(&mut conn, created.id);
        assert!(result.is_ok());

        // Verify it's gone
        let find_result = AutoApplyConfigService::find_by_id(&mut conn, created.id);
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
            AutoApplyConfigService::create(&mut conn, dto, "test_user").unwrap();
        }

        let sort_specs: &[&str] = &[
            "userId,asc",
            "userId,desc",
            "mode,asc",
            "mode,desc",
            "minScore,asc",
            "minScore,desc",
            "maxPerDay,asc",
            "maxPerDay,desc",
            "sources,asc",
            "sources,desc",
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
            let result = AutoApplyConfigService::find_all(&mut conn, &page_request);
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
