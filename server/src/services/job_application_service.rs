use chrono::Utc;
use diesel::prelude::*;

use crate::db::connection::DbConnection;
use crate::db::schema::job_application;
use crate::db::schema::job_offer;
use crate::db::schema::candidate_profile;
use crate::dto::{CreateJobApplicationDto, PageRequest, UpdateJobApplicationDto};
use crate::errors::AppError;
use crate::models::{NewJobApplication, UpdateJobApplication, JobApplication, JobOffer, CandidateProfile};

pub struct JobApplicationService;

impl JobApplicationService {
    /// Find all jobApplications with pagination
    pub fn find_all(
        conn: &mut DbConnection,
        page_request: &PageRequest,
    ) -> Result<(Vec<JobApplication>, i64), AppError> {
        let page = page_request.page.unwrap_or(0);
        let size = page_request.size.unwrap_or(20).min(100);
        let offset = page * size;

        let total: i64 = job_application::table
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
                    job_application::table
                        .order(job_application::user_id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::user_id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
            "status" => {
                if is_desc {
                    job_application::table
                        .order(job_application::status.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::status.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
            "coverLetter" | "cover_letter" => {
                if is_desc {
                    job_application::table
                        .order(job_application::cover_letter.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::cover_letter.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
            "notes" => {
                if is_desc {
                    job_application::table
                        .order(job_application::notes.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::notes.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
            "matchScore" | "match_score" => {
                if is_desc {
                    job_application::table
                        .order(job_application::match_score.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::match_score.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
            "createdAt" | "created_at" => {
                if is_desc {
                    job_application::table
                        .order(job_application::created_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::created_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
            "updatedAt" | "updated_at" => {
                if is_desc {
                    job_application::table
                        .order(job_application::updated_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::updated_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
            "appliedAt" | "applied_at" => {
                if is_desc {
                    job_application::table
                        .order(job_application::applied_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::applied_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
            "createdDate" | "created_date" => {
                if is_desc {
                    job_application::table
                        .order(job_application::created_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::created_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
            "lastModifiedDate" | "last_modified_date" => {
                if is_desc {
                    job_application::table
                        .order(job_application::last_modified_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::last_modified_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
            _ => {
                // Default: sort by id
                if is_desc {
                    job_application::table
                        .order(job_application::id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                } else {
                    job_application::table
                        .order(job_application::id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobApplication::as_select()).load(conn)
                }
            }
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((results, total))
    }

    /// Find jobApplication by ID
    pub fn find_by_id(conn: &mut DbConnection, id: i32) -> Result<JobApplication, AppError> {
        job_application::table
            .find(id)
            .select(JobApplication::as_select()).first(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound(format!("JobApplication {} not found", id))
                }
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Create a new jobApplication
    pub fn create(
        conn: &mut DbConnection,
        dto: CreateJobApplicationDto,
        created_by: &str,
    ) -> Result<JobApplication, AppError> {
        let now = Utc::now().naive_utc();

        let new_entity = NewJobApplication {
            user_id: dto.user_id,
            status: dto.status,
            cover_letter: dto.cover_letter,
            notes: dto.notes,
            match_score: dto.match_score,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            applied_at: dto.applied_at,
            jobOffer_id: dto.jobOffer_id,
            candidateProfile_id: dto.candidateProfile_id,
            created_by: Some(created_by.to_string()),
            created_date: Some(now),
            last_modified_by: Some(created_by.to_string()),
            last_modified_date: Some(now),
        };

        let entity = diesel::insert_into(job_application::table)
            .values(&new_entity)
            .get_result::<JobApplication>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(entity)
    }

    /// Update an existing jobApplication
    pub fn update(
        conn: &mut DbConnection,
        id: i32,
        dto: UpdateJobApplicationDto,
        modified_by: &str,
    ) -> Result<JobApplication, AppError> {
        let now = Utc::now().naive_utc();

        let update = UpdateJobApplication {
            user_id: dto.user_id,
            status: dto.status,
            cover_letter: dto.cover_letter,
            notes: dto.notes,
            match_score: dto.match_score,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            applied_at: dto.applied_at,
            jobOffer_id: dto.jobOffer_id.map(Some),
            candidateProfile_id: dto.candidateProfile_id.map(Some),
            last_modified_by: Some(modified_by.to_string()),
            last_modified_date: Some(now),
        };

        diesel::update(job_application::table.find(id))
            .set(&update)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Self::find_by_id(conn, id)
    }

    /// Delete a jobApplication
    pub fn delete(conn: &mut DbConnection, id: i32) -> Result<(), AppError> {
        diesel::delete(job_application::table.find(id))
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Find related jobOffer by ID
    pub fn find_jobOffer_by_id(conn: &mut DbConnection, id: i32) -> Result<Option<JobOffer>, AppError> {
        job_offer::table
            .find(id)
            .select(JobOffer::as_select()).first(conn)
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    /// Find related candidateProfile by ID
    pub fn find_candidateProfile_by_id(conn: &mut DbConnection, id: i32) -> Result<Option<CandidateProfile>, AppError> {
        candidate_profile::table
            .find(id)
            .select(CandidateProfile::as_select()).first(conn)
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

    fn create_test_dto_with_suffix(suffix: u32) -> CreateJobApplicationDto {
        let _ = suffix; // Suppress unused warning when there are no String fields
        CreateJobApplicationDto {
            user_id: format!("test_value_{}", suffix),
            status: None,
            cover_letter: None,
            notes: None,
            match_score: None,
            created_at: None,
            updated_at: None,
            applied_at: None,
            jobOffer_id: None,
            candidateProfile_id: None,
        }
    }

    fn create_test_dto() -> CreateJobApplicationDto {
        let suffix = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        create_test_dto_with_suffix(suffix)
    }

    #[test]
    fn test_create_jobApplication() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let dto = create_test_dto();
        let result = JobApplicationService::create(&mut conn, dto, "test_user");

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
        let created = JobApplicationService::create(&mut conn, dto, "test_user").unwrap();

        // Find it by ID
        let result = JobApplicationService::find_by_id(&mut conn, created.id);
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let result = JobApplicationService::find_by_id(&mut conn, 99999);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_all() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create a few entities with unique values
        for _ in 0..3 {
            let dto = create_test_dto();
            JobApplicationService::create(&mut conn, dto, "test_user").unwrap();
        }

        let page_request = PageRequest {
            page: Some(0),
            size: Some(10),
            sort: Vec::new(),
        };

        let result = JobApplicationService::find_all(&mut conn, &page_request);
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
            JobApplicationService::create(&mut conn, dto, "test_user").unwrap();
        }

        // Request page 0 with size 2
        let page_request = PageRequest {
            page: Some(0),
            size: Some(2),
            sort: Vec::new(),
        };

        let result = JobApplicationService::find_all(&mut conn, &page_request);
        assert!(result.is_ok());
        let (items, _total) = result.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_update_jobApplication() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = JobApplicationService::create(&mut conn, dto, "test_user").unwrap();

        // Update it
        let update_dto = UpdateJobApplicationDto {
            user_id: Some("updated_value".to_string()),
            status: Some("updated_value".to_string()),
            cover_letter: Some("updated_value".to_string()),
            notes: Some("updated_value".to_string()),
            match_score: Some(999.99),
            created_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            updated_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            applied_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            jobOffer_id: None,
            candidateProfile_id: None,
        };

        let result = JobApplicationService::update(&mut conn, created.id, update_dto, "test_user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_jobApplication() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = JobApplicationService::create(&mut conn, dto, "test_user").unwrap();

        // Delete it
        let result = JobApplicationService::delete(&mut conn, created.id);
        assert!(result.is_ok());

        // Verify it's gone
        let find_result = JobApplicationService::find_by_id(&mut conn, created.id);
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
            JobApplicationService::create(&mut conn, dto, "test_user").unwrap();
        }

        let sort_specs: &[&str] = &[
            "userId,asc",
            "userId,desc",
            "status,asc",
            "status,desc",
            "coverLetter,asc",
            "coverLetter,desc",
            "notes,asc",
            "notes,desc",
            "matchScore,asc",
            "matchScore,desc",
            "createdAt,asc",
            "createdAt,desc",
            "updatedAt,asc",
            "updatedAt,desc",
            "appliedAt,asc",
            "appliedAt,desc",
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
            let result = JobApplicationService::find_all(&mut conn, &page_request);
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

    // Phase 2d (2026-06-07): cover find_jobOffer_by_id, generated
    // only when this entity has a many-to-one relationship to
    // JobOffer. The function returns Result<Option<_>, _> —
    // both the Some and None paths exist in template code; pin both.
    #[test]
    fn test_find_jobOffer_by_id_returns_none_for_nonexistent() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();
        // i32::MAX is a valid id type but vanishingly unlikely to exist in
        // a freshly-migrated test DB.
        let result = JobApplicationService::find_jobOffer_by_id(&mut conn, i32::MAX);
        assert!(matches!(result, Ok(None)), "expected Ok(None), got {:?}", result);
    }

    // Phase 2d (2026-06-07): cover find_candidateProfile_by_id, generated
    // only when this entity has a many-to-one relationship to
    // CandidateProfile. The function returns Result<Option<_>, _> —
    // both the Some and None paths exist in template code; pin both.
    #[test]
    fn test_find_candidateProfile_by_id_returns_none_for_nonexistent() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();
        // i32::MAX is a valid id type but vanishingly unlikely to exist in
        // a freshly-migrated test DB.
        let result = JobApplicationService::find_candidateProfile_by_id(&mut conn, i32::MAX);
        assert!(matches!(result, Ok(None)), "expected Ok(None), got {:?}", result);
    }
}
