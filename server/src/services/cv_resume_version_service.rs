use chrono::Utc;
use diesel::prelude::*;

use crate::db::connection::DbConnection;
use crate::db::schema::cv_resume_version;
use crate::db::schema::cv_resume;
use crate::dto::{CreateCvResumeVersionDto, PageRequest, UpdateCvResumeVersionDto};
use crate::errors::AppError;
use crate::models::{NewCvResumeVersion, UpdateCvResumeVersion, CvResumeVersion, CvResume};
use uuid::Uuid;

pub struct CvResumeVersionService;

impl CvResumeVersionService {
    /// Find all cvResumeVersions with pagination
    pub fn find_all(
        conn: &mut DbConnection,
        page_request: &PageRequest,
    ) -> Result<(Vec<CvResumeVersion>, i64), AppError> {
        let page = page_request.page.unwrap_or(0);
        let size = page_request.size.unwrap_or(20).min(100);
        let offset = page * size;

        let total: i64 = cv_resume_version::table
            .count()
            .get_result(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Get primary sort parameter (format: "field,direction" e.g., "name,asc" or "cost,desc")
        let (sort_field, sort_dir) = page_request.primary_sort().unwrap_or(("id", "asc"));

        let is_desc = sort_dir.eq_ignore_ascii_case("desc");

        // Dynamic sorting based on field name
        let results = match sort_field {
            "versionNumber" | "version_number" => {
                if is_desc {
                    cv_resume_version::table
                        .order(cv_resume_version::version_number.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                } else {
                    cv_resume_version::table
                        .order(cv_resume_version::version_number.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                }
            }
            "title" => {
                if is_desc {
                    cv_resume_version::table
                        .order(cv_resume_version::title.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                } else {
                    cv_resume_version::table
                        .order(cv_resume_version::title.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                }
            }
            "template" => {
                if is_desc {
                    cv_resume_version::table
                        .order(cv_resume_version::template.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                } else {
                    cv_resume_version::table
                        .order(cv_resume_version::template.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                }
            }
            "data" => {
                if is_desc {
                    cv_resume_version::table
                        .order(cv_resume_version::data.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                } else {
                    cv_resume_version::table
                        .order(cv_resume_version::data.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                }
            }
            "createdAt" | "created_at" => {
                if is_desc {
                    cv_resume_version::table
                        .order(cv_resume_version::created_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                } else {
                    cv_resume_version::table
                        .order(cv_resume_version::created_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                }
            }
            "createdDate" | "created_date" => {
                if is_desc {
                    cv_resume_version::table
                        .order(cv_resume_version::created_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                } else {
                    cv_resume_version::table
                        .order(cv_resume_version::created_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                }
            }
            "lastModifiedDate" | "last_modified_date" => {
                if is_desc {
                    cv_resume_version::table
                        .order(cv_resume_version::last_modified_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                } else {
                    cv_resume_version::table
                        .order(cv_resume_version::last_modified_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                }
            }
            _ => {
                // Default: sort by id
                if is_desc {
                    cv_resume_version::table
                        .order(cv_resume_version::id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                } else {
                    cv_resume_version::table
                        .order(cv_resume_version::id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResumeVersion::as_select()).load(conn)
                }
            }
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((results, total))
    }

    /// Find cvResumeVersion by ID
    pub fn find_by_id(conn: &mut DbConnection, id: Uuid) -> Result<CvResumeVersion, AppError> {
        cv_resume_version::table
            .find(id)
            .select(CvResumeVersion::as_select()).first(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound(format!("CvResumeVersion {} not found", id))
                }
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Create a new cvResumeVersion
    pub fn create(
        conn: &mut DbConnection,
        dto: CreateCvResumeVersionDto,
        created_by: &str,
    ) -> Result<CvResumeVersion, AppError> {
        let now = Utc::now().naive_utc();

        let new_entity = NewCvResumeVersion {
            version_number: dto.version_number,
            title: dto.title,
            template: dto.template,
            data: dto.data,
            created_at: dto.created_at,
            resume_id: dto.resume_id,
            created_by: Some(created_by.to_string()),
            created_date: Some(now),
            last_modified_by: Some(created_by.to_string()),
            last_modified_date: Some(now),
        };

        let entity = diesel::insert_into(cv_resume_version::table)
            .values(&new_entity)
            .get_result::<CvResumeVersion>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(entity)
    }

    /// Update an existing cvResumeVersion
    pub fn update(
        conn: &mut DbConnection,
        id: Uuid,
        dto: UpdateCvResumeVersionDto,
        modified_by: &str,
    ) -> Result<CvResumeVersion, AppError> {
        let now = Utc::now().naive_utc();

        let update = UpdateCvResumeVersion {
            version_number: dto.version_number,
            title: dto.title,
            template: dto.template,
            data: dto.data,
            created_at: dto.created_at,
            resume_id: dto.resume_id.map(Some),
            last_modified_by: Some(modified_by.to_string()),
            last_modified_date: Some(now),
        };

        diesel::update(cv_resume_version::table.find(id))
            .set(&update)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Self::find_by_id(conn, id)
    }

    /// Delete a cvResumeVersion
    pub fn delete(conn: &mut DbConnection, id: Uuid) -> Result<(), AppError> {
        diesel::delete(cv_resume_version::table.find(id))
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Find related resume by ID
    pub fn find_resume_by_id(conn: &mut DbConnection, id: Uuid) -> Result<Option<CvResume>, AppError> {
        cv_resume::table
            .find(id)
            .select(CvResume::as_select()).first(conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_pool;
    use crate::dto::PageRequest;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn create_test_dto_with_suffix(suffix: u32) -> CreateCvResumeVersionDto {
        let _ = suffix; // Suppress unused warning when there are no String fields
        CreateCvResumeVersionDto {
            version_number: 1,
            title: None,
            template: None,
            data: format!("test_value_{}", suffix),
            created_at: None,
            resume_id: None,
        }
    }

    fn create_test_dto() -> CreateCvResumeVersionDto {
        let suffix = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        create_test_dto_with_suffix(suffix)
    }

    #[test]
    fn test_create_cvResumeVersion() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let dto = create_test_dto();
        let result = CvResumeVersionService::create(&mut conn, dto, "test_user");

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
        let created = CvResumeVersionService::create(&mut conn, dto, "test_user").unwrap();

        // Find it by ID
        let result = CvResumeVersionService::find_by_id(&mut conn, created.id);
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let result = CvResumeVersionService::find_by_id(&mut conn, Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn test_find_all() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create a few entities with unique values
        for _ in 0..3 {
            let dto = create_test_dto();
            CvResumeVersionService::create(&mut conn, dto, "test_user").unwrap();
        }

        let page_request = PageRequest {
            page: Some(0),
            size: Some(10),
            sort: Vec::new(),
        };

        let result = CvResumeVersionService::find_all(&mut conn, &page_request);
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
            CvResumeVersionService::create(&mut conn, dto, "test_user").unwrap();
        }

        // Request page 0 with size 2
        let page_request = PageRequest {
            page: Some(0),
            size: Some(2),
            sort: Vec::new(),
        };

        let result = CvResumeVersionService::find_all(&mut conn, &page_request);
        assert!(result.is_ok());
        let (items, _total) = result.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_update_cvResumeVersion() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = CvResumeVersionService::create(&mut conn, dto, "test_user").unwrap();

        // Update it
        let update_dto = UpdateCvResumeVersionDto {
            version_number: Some(999),
            title: Some("updated_value".to_string()),
            template: Some("updated_value".to_string()),
            data: Some("updated_value".to_string()),
            created_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            resume_id: None,
        };

        let result = CvResumeVersionService::update(&mut conn, created.id, update_dto, "test_user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_cvResumeVersion() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = CvResumeVersionService::create(&mut conn, dto, "test_user").unwrap();

        // Delete it
        let result = CvResumeVersionService::delete(&mut conn, created.id);
        assert!(result.is_ok());

        // Verify it's gone
        let find_result = CvResumeVersionService::find_by_id(&mut conn, created.id);
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
            CvResumeVersionService::create(&mut conn, dto, "test_user").unwrap();
        }

        let sort_specs: &[&str] = &[
            "versionNumber,asc",
            "versionNumber,desc",
            "title,asc",
            "title,desc",
            "template,asc",
            "template,desc",
            "data,asc",
            "data,desc",
            "createdAt,asc",
            "createdAt,desc",
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
            let result = CvResumeVersionService::find_all(&mut conn, &page_request);
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

    // Phase 2d (2026-06-07): cover find_resume_by_id, generated
    // only when this entity has a many-to-one relationship to
    // CvResume. The function returns Result<Option<_>, _> —
    // both the Some and None paths exist in template code; pin both.
    #[test]
    fn test_find_resume_by_id_returns_none_for_nonexistent() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();
        // Uuid::new_v4() is a valid id type but vanishingly unlikely to exist in
        // a freshly-migrated test DB.
        let result = CvResumeVersionService::find_resume_by_id(&mut conn, Uuid::new_v4());
        assert!(matches!(result, Ok(None)), "expected Ok(None), got {:?}", result);
    }
}
