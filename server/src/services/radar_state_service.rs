use chrono::Utc;
use diesel::prelude::*;

use crate::db::connection::DbConnection;
use crate::db::schema::radar_state;
use crate::dto::{CreateRadarStateDto, PageRequest, UpdateRadarStateDto};
use crate::errors::AppError;
use crate::models::{NewRadarState, UpdateRadarState, RadarState};
use uuid::Uuid;

pub struct RadarStateService;

impl RadarStateService {
    /// Find all radarStates with pagination
    pub fn find_all(
        conn: &mut DbConnection,
        page_request: &PageRequest,
    ) -> Result<(Vec<RadarState>, i64), AppError> {
        let page = page_request.page.unwrap_or(0);
        let size = page_request.size.unwrap_or(20).min(100);
        let offset = page * size;

        let total: i64 = radar_state::table
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
                    radar_state::table
                        .order(radar_state::user_id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(RadarState::as_select()).load(conn)
                } else {
                    radar_state::table
                        .order(radar_state::user_id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(RadarState::as_select()).load(conn)
                }
            }
            "lastOfferAt" | "last_offer_at" => {
                if is_desc {
                    radar_state::table
                        .order(radar_state::last_offer_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(RadarState::as_select()).load(conn)
                } else {
                    radar_state::table
                        .order(radar_state::last_offer_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(RadarState::as_select()).load(conn)
                }
            }
            "createdDate" | "created_date" => {
                if is_desc {
                    radar_state::table
                        .order(radar_state::created_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(RadarState::as_select()).load(conn)
                } else {
                    radar_state::table
                        .order(radar_state::created_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(RadarState::as_select()).load(conn)
                }
            }
            "lastModifiedDate" | "last_modified_date" => {
                if is_desc {
                    radar_state::table
                        .order(radar_state::last_modified_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(RadarState::as_select()).load(conn)
                } else {
                    radar_state::table
                        .order(radar_state::last_modified_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(RadarState::as_select()).load(conn)
                }
            }
            _ => {
                // Default: sort by id
                if is_desc {
                    radar_state::table
                        .order(radar_state::id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(RadarState::as_select()).load(conn)
                } else {
                    radar_state::table
                        .order(radar_state::id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(RadarState::as_select()).load(conn)
                }
            }
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((results, total))
    }

    /// Find radarState by ID
    pub fn find_by_id(conn: &mut DbConnection, id: Uuid) -> Result<RadarState, AppError> {
        radar_state::table
            .find(id)
            .select(RadarState::as_select()).first(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound(format!("RadarState {} not found", id))
                }
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Create a new radarState
    pub fn create(
        conn: &mut DbConnection,
        dto: CreateRadarStateDto,
        created_by: &str,
    ) -> Result<RadarState, AppError> {
        let now = Utc::now().naive_utc();

        let new_entity = NewRadarState {
            user_id: dto.user_id,
            last_offer_at: dto.last_offer_at,
            created_by: Some(created_by.to_string()),
            created_date: Some(now),
            last_modified_by: Some(created_by.to_string()),
            last_modified_date: Some(now),
        };

        let entity = diesel::insert_into(radar_state::table)
            .values(&new_entity)
            .get_result::<RadarState>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(entity)
    }

    /// Update an existing radarState
    pub fn update(
        conn: &mut DbConnection,
        id: Uuid,
        dto: UpdateRadarStateDto,
        modified_by: &str,
    ) -> Result<RadarState, AppError> {
        let now = Utc::now().naive_utc();

        let update = UpdateRadarState {
            user_id: dto.user_id,
            last_offer_at: dto.last_offer_at,
            last_modified_by: Some(modified_by.to_string()),
            last_modified_date: Some(now),
        };

        diesel::update(radar_state::table.find(id))
            .set(&update)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Self::find_by_id(conn, id)
    }

    /// Delete a radarState
    pub fn delete(conn: &mut DbConnection, id: Uuid) -> Result<(), AppError> {
        diesel::delete(radar_state::table.find(id))
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

    fn create_test_dto_with_suffix(suffix: u32) -> CreateRadarStateDto {
        let _ = suffix; // Suppress unused warning when there are no String fields
        CreateRadarStateDto {
            user_id: format!("test_value_{}", suffix),
            last_offer_at: None,
        }
    }

    fn create_test_dto() -> CreateRadarStateDto {
        let suffix = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        create_test_dto_with_suffix(suffix)
    }

    #[test]
    fn test_create_radarState() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let dto = create_test_dto();
        let result = RadarStateService::create(&mut conn, dto, "test_user");

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
        let created = RadarStateService::create(&mut conn, dto, "test_user").unwrap();

        // Find it by ID
        let result = RadarStateService::find_by_id(&mut conn, created.id);
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let result = RadarStateService::find_by_id(&mut conn, Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn test_find_all() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create a few entities with unique values
        for _ in 0..3 {
            let dto = create_test_dto();
            RadarStateService::create(&mut conn, dto, "test_user").unwrap();
        }

        let page_request = PageRequest {
            page: Some(0),
            size: Some(10),
            sort: Vec::new(),
        };

        let result = RadarStateService::find_all(&mut conn, &page_request);
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
            RadarStateService::create(&mut conn, dto, "test_user").unwrap();
        }

        // Request page 0 with size 2
        let page_request = PageRequest {
            page: Some(0),
            size: Some(2),
            sort: Vec::new(),
        };

        let result = RadarStateService::find_all(&mut conn, &page_request);
        assert!(result.is_ok());
        let (items, _total) = result.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_update_radarState() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = RadarStateService::create(&mut conn, dto, "test_user").unwrap();

        // Update it
        let update_dto = UpdateRadarStateDto {
            user_id: Some("updated_value".to_string()),
            last_offer_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
        };

        let result = RadarStateService::update(&mut conn, created.id, update_dto, "test_user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_radarState() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = RadarStateService::create(&mut conn, dto, "test_user").unwrap();

        // Delete it
        let result = RadarStateService::delete(&mut conn, created.id);
        assert!(result.is_ok());

        // Verify it's gone
        let find_result = RadarStateService::find_by_id(&mut conn, created.id);
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
            RadarStateService::create(&mut conn, dto, "test_user").unwrap();
        }

        let sort_specs: &[&str] = &[
            "userId,asc",
            "userId,desc",
            "lastOfferAt,asc",
            "lastOfferAt,desc",
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
            let result = RadarStateService::find_all(&mut conn, &page_request);
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
