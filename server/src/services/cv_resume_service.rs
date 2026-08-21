use chrono::Utc;
use diesel::prelude::*;

use crate::db::connection::DbConnection;
use crate::db::schema::cv_resume;
use crate::dto::{CreateCvResumeDto, PageRequest, UpdateCvResumeDto};
use crate::errors::AppError;
use crate::models::{NewCvResume, UpdateCvResume, CvResume};

pub struct CvResumeService;

impl CvResumeService {
    /// Find all cvResumes with pagination
    pub fn find_all(
        conn: &mut DbConnection,
        page_request: &PageRequest,
    ) -> Result<(Vec<CvResume>, i64), AppError> {
        let page = page_request.page.unwrap_or(0);
        let size = page_request.size.unwrap_or(20).min(100);
        let offset = page * size;

        let total: i64 = cv_resume::table
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
                    cv_resume::table
                        .order(cv_resume::user_id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                } else {
                    cv_resume::table
                        .order(cv_resume::user_id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                }
            }
            "title" => {
                if is_desc {
                    cv_resume::table
                        .order(cv_resume::title.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                } else {
                    cv_resume::table
                        .order(cv_resume::title.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                }
            }
            "template" => {
                if is_desc {
                    cv_resume::table
                        .order(cv_resume::template.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                } else {
                    cv_resume::table
                        .order(cv_resume::template.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                }
            }
            "data" => {
                if is_desc {
                    cv_resume::table
                        .order(cv_resume::data.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                } else {
                    cv_resume::table
                        .order(cv_resume::data.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                }
            }
            "versionNumber" | "version_number" => {
                if is_desc {
                    cv_resume::table
                        .order(cv_resume::version_number.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                } else {
                    cv_resume::table
                        .order(cv_resume::version_number.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                }
            }
            "createdAt" | "created_at" => {
                if is_desc {
                    cv_resume::table
                        .order(cv_resume::created_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                } else {
                    cv_resume::table
                        .order(cv_resume::created_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                }
            }
            "updatedAt" | "updated_at" => {
                if is_desc {
                    cv_resume::table
                        .order(cv_resume::updated_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                } else {
                    cv_resume::table
                        .order(cv_resume::updated_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                }
            }
            "createdDate" | "created_date" => {
                if is_desc {
                    cv_resume::table
                        .order(cv_resume::created_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                } else {
                    cv_resume::table
                        .order(cv_resume::created_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                }
            }
            "lastModifiedDate" | "last_modified_date" => {
                if is_desc {
                    cv_resume::table
                        .order(cv_resume::last_modified_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                } else {
                    cv_resume::table
                        .order(cv_resume::last_modified_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                }
            }
            _ => {
                // Default: sort by id
                if is_desc {
                    cv_resume::table
                        .order(cv_resume::id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                } else {
                    cv_resume::table
                        .order(cv_resume::id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CvResume::as_select()).load(conn)
                }
            }
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((results, total))
    }

    /// Find cvResume by ID
    pub fn find_by_id(conn: &mut DbConnection, id: i32) -> Result<CvResume, AppError> {
        cv_resume::table
            .find(id)
            .select(CvResume::as_select()).first(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound(format!("CvResume {} not found", id))
                }
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Create a new cvResume
    pub fn create(
        conn: &mut DbConnection,
        dto: CreateCvResumeDto,
        created_by: &str,
    ) -> Result<CvResume, AppError> {
        let now = Utc::now().naive_utc();

        let new_entity = NewCvResume {
            user_id: dto.user_id,
            title: dto.title,
            template: dto.template,
            data: dto.data,
            version_number: dto.version_number,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            created_by: Some(created_by.to_string()),
            created_date: Some(now),
            last_modified_by: Some(created_by.to_string()),
            last_modified_date: Some(now),
        };

        let entity = diesel::insert_into(cv_resume::table)
            .values(&new_entity)
            .get_result::<CvResume>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(entity)
    }

    /// Update an existing cvResume
    pub fn update(
        conn: &mut DbConnection,
        id: i32,
        dto: UpdateCvResumeDto,
        modified_by: &str,
    ) -> Result<CvResume, AppError> {
        let now = Utc::now().naive_utc();

        let update = UpdateCvResume {
            user_id: dto.user_id,
            title: dto.title,
            template: dto.template,
            data: dto.data,
            version_number: dto.version_number,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            last_modified_by: Some(modified_by.to_string()),
            last_modified_date: Some(now),
        };

        diesel::update(cv_resume::table.find(id))
            .set(&update)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Self::find_by_id(conn, id)
    }

    /// Delete a cvResume
    pub fn delete(conn: &mut DbConnection, id: i32) -> Result<(), AppError> {
        diesel::delete(cv_resume::table.find(id))
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

    fn create_test_dto_with_suffix(suffix: u32) -> CreateCvResumeDto {
        let _ = suffix; // Suppress unused warning when there are no String fields
        CreateCvResumeDto {
            user_id: format!("test_value_{}", suffix),
            title: None,
            template: None,
            data: format!("test_value_{}", suffix),
            version_number: 1,
            created_at: None,
            updated_at: None,
        }
    }

    fn create_test_dto() -> CreateCvResumeDto {
        let suffix = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        create_test_dto_with_suffix(suffix)
    }

    #[test]
    fn test_create_cvResume() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let dto = create_test_dto();
        let result = CvResumeService::create(&mut conn, dto, "test_user");

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
        let created = CvResumeService::create(&mut conn, dto, "test_user").unwrap();

        // Find it by ID
        let result = CvResumeService::find_by_id(&mut conn, created.id);
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let result = CvResumeService::find_by_id(&mut conn, 99999);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_all() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create a few entities with unique values
        for _ in 0..3 {
            let dto = create_test_dto();
            CvResumeService::create(&mut conn, dto, "test_user").unwrap();
        }

        let page_request = PageRequest {
            page: Some(0),
            size: Some(10),
            sort: Vec::new(),
        };

        let result = CvResumeService::find_all(&mut conn, &page_request);
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
            CvResumeService::create(&mut conn, dto, "test_user").unwrap();
        }

        // Request page 0 with size 2
        let page_request = PageRequest {
            page: Some(0),
            size: Some(2),
            sort: Vec::new(),
        };

        let result = CvResumeService::find_all(&mut conn, &page_request);
        assert!(result.is_ok());
        let (items, _total) = result.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_update_cvResume() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = CvResumeService::create(&mut conn, dto, "test_user").unwrap();

        // Update it
        let update_dto = UpdateCvResumeDto {
            user_id: Some("updated_value".to_string()),
            title: Some("updated_value".to_string()),
            template: Some("updated_value".to_string()),
            data: Some("updated_value".to_string()),
            version_number: Some(999),
            created_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            updated_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
        };

        let result = CvResumeService::update(&mut conn, created.id, update_dto, "test_user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_cvResume() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = CvResumeService::create(&mut conn, dto, "test_user").unwrap();

        // Delete it
        let result = CvResumeService::delete(&mut conn, created.id);
        assert!(result.is_ok());

        // Verify it's gone
        let find_result = CvResumeService::find_by_id(&mut conn, created.id);
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
            CvResumeService::create(&mut conn, dto, "test_user").unwrap();
        }

        let sort_specs: &[&str] = &[
            "userId,asc",
            "userId,desc",
            "title,asc",
            "title,desc",
            "template,asc",
            "template,desc",
            "data,asc",
            "data,desc",
            "versionNumber,asc",
            "versionNumber,desc",
            "createdAt,asc",
            "createdAt,desc",
            "updatedAt,asc",
            "updatedAt,desc",
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
            let result = CvResumeService::find_all(&mut conn, &page_request);
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
