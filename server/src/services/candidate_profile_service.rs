use chrono::Utc;
use diesel::prelude::*;

use crate::db::connection::DbConnection;
use crate::db::schema::candidate_profile;
use crate::dto::{CreateCandidateProfileDto, PageRequest, UpdateCandidateProfileDto};
use crate::errors::AppError;
use crate::models::{NewCandidateProfile, UpdateCandidateProfile, CandidateProfile};

pub struct CandidateProfileService;

impl CandidateProfileService {
    /// Find all candidateProfiles with pagination
    pub fn find_all(
        conn: &mut DbConnection,
        page_request: &PageRequest,
    ) -> Result<(Vec<CandidateProfile>, i64), AppError> {
        let page = page_request.page.unwrap_or(0);
        let size = page_request.size.unwrap_or(20).min(100);
        let offset = page * size;

        let total: i64 = candidate_profile::table
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
                    candidate_profile::table
                        .order(candidate_profile::user_id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::user_id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "fullName" | "full_name" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::full_name.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::full_name.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "email" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::email.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::email.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "location" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::location.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::location.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "yearsOfExperience" | "years_of_experience" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::years_of_experience.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::years_of_experience.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "skills" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::skills.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::skills.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "experiences" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::experiences.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::experiences.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "preferredRoles" | "preferred_roles" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::preferred_roles.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::preferred_roles.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "languages" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::languages.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::languages.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "education" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::education.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::education.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "certifications" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::certifications.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::certifications.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "rawMarkdown" | "raw_markdown" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::raw_markdown.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::raw_markdown.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "cvFilename" | "cv_filename" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::cv_filename.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::cv_filename.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "embeddingModel" | "embedding_model" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::embedding_model.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::embedding_model.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "embeddedAt" | "embedded_at" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::embedded_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::embedded_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "createdAt" | "created_at" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::created_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::created_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "updatedAt" | "updated_at" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::updated_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::updated_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "createdDate" | "created_date" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::created_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::created_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            "lastModifiedDate" | "last_modified_date" => {
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::last_modified_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::last_modified_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
            _ => {
                // Default: sort by id
                if is_desc {
                    candidate_profile::table
                        .order(candidate_profile::id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                } else {
                    candidate_profile::table
                        .order(candidate_profile::id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(CandidateProfile::as_select()).load(conn)
                }
            }
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((results, total))
    }

    /// Find candidateProfile by ID
    pub fn find_by_id(conn: &mut DbConnection, id: i32) -> Result<CandidateProfile, AppError> {
        candidate_profile::table
            .find(id)
            .select(CandidateProfile::as_select()).first(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound(format!("CandidateProfile {} not found", id))
                }
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Create a new candidateProfile
    pub fn create(
        conn: &mut DbConnection,
        dto: CreateCandidateProfileDto,
        created_by: &str,
    ) -> Result<CandidateProfile, AppError> {
        let now = Utc::now().naive_utc();

        let new_entity = NewCandidateProfile {
            user_id: dto.user_id,
            full_name: dto.full_name,
            email: dto.email,
            location: dto.location,
            years_of_experience: dto.years_of_experience,
            skills: dto.skills,
            experiences: dto.experiences,
            preferred_roles: dto.preferred_roles,
            languages: dto.languages,
            education: dto.education,
            certifications: dto.certifications,
            raw_markdown: dto.raw_markdown,
            cv_filename: dto.cv_filename,
            embedding_model: dto.embedding_model,
            embedded_at: dto.embedded_at,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            created_by: Some(created_by.to_string()),
            created_date: Some(now),
            last_modified_by: Some(created_by.to_string()),
            last_modified_date: Some(now),
        };

        let entity = diesel::insert_into(candidate_profile::table)
            .values(&new_entity)
            .get_result::<CandidateProfile>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(entity)
    }

    /// Update an existing candidateProfile
    pub fn update(
        conn: &mut DbConnection,
        id: i32,
        dto: UpdateCandidateProfileDto,
        modified_by: &str,
    ) -> Result<CandidateProfile, AppError> {
        let now = Utc::now().naive_utc();

        let update = UpdateCandidateProfile {
            user_id: dto.user_id,
            full_name: dto.full_name,
            email: dto.email,
            location: dto.location,
            years_of_experience: dto.years_of_experience,
            skills: dto.skills,
            experiences: dto.experiences,
            preferred_roles: dto.preferred_roles,
            languages: dto.languages,
            education: dto.education,
            certifications: dto.certifications,
            raw_markdown: dto.raw_markdown,
            cv_filename: dto.cv_filename,
            embedding_model: dto.embedding_model,
            embedded_at: dto.embedded_at,
            created_at: dto.created_at,
            updated_at: dto.updated_at,
            last_modified_by: Some(modified_by.to_string()),
            last_modified_date: Some(now),
        };

        diesel::update(candidate_profile::table.find(id))
            .set(&update)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Self::find_by_id(conn, id)
    }

    /// Delete a candidateProfile
    pub fn delete(conn: &mut DbConnection, id: i32) -> Result<(), AppError> {
        diesel::delete(candidate_profile::table.find(id))
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

    fn create_test_dto_with_suffix(suffix: u32) -> CreateCandidateProfileDto {
        let _ = suffix; // Suppress unused warning when there are no String fields
        CreateCandidateProfileDto {
            user_id: format!("test_value_{}", suffix),
            full_name: None,
            email: None,
            location: None,
            years_of_experience: None,
            skills: None,
            experiences: None,
            preferred_roles: None,
            languages: None,
            education: None,
            certifications: None,
            raw_markdown: None,
            cv_filename: None,
            embedding_model: None,
            embedded_at: None,
            created_at: None,
            updated_at: None,
        }
    }

    fn create_test_dto() -> CreateCandidateProfileDto {
        let suffix = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        create_test_dto_with_suffix(suffix)
    }

    #[test]
    fn test_create_candidateProfile() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let dto = create_test_dto();
        let result = CandidateProfileService::create(&mut conn, dto, "test_user");

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
        let created = CandidateProfileService::create(&mut conn, dto, "test_user").unwrap();

        // Find it by ID
        let result = CandidateProfileService::find_by_id(&mut conn, created.id);
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let result = CandidateProfileService::find_by_id(&mut conn, 99999);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_all() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create a few entities with unique values
        for _ in 0..3 {
            let dto = create_test_dto();
            CandidateProfileService::create(&mut conn, dto, "test_user").unwrap();
        }

        let page_request = PageRequest {
            page: Some(0),
            size: Some(10),
            sort: Vec::new(),
        };

        let result = CandidateProfileService::find_all(&mut conn, &page_request);
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
            CandidateProfileService::create(&mut conn, dto, "test_user").unwrap();
        }

        // Request page 0 with size 2
        let page_request = PageRequest {
            page: Some(0),
            size: Some(2),
            sort: Vec::new(),
        };

        let result = CandidateProfileService::find_all(&mut conn, &page_request);
        assert!(result.is_ok());
        let (items, _total) = result.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_update_candidateProfile() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = CandidateProfileService::create(&mut conn, dto, "test_user").unwrap();

        // Update it
        let update_dto = UpdateCandidateProfileDto {
            user_id: Some("updated_value".to_string()),
            full_name: Some("updated_value".to_string()),
            email: Some("updated_value".to_string()),
            location: Some("updated_value".to_string()),
            years_of_experience: Some(999),
            skills: Some("updated_value".to_string()),
            experiences: Some("updated_value".to_string()),
            preferred_roles: Some("updated_value".to_string()),
            languages: Some("updated_value".to_string()),
            education: Some("updated_value".to_string()),
            certifications: Some("updated_value".to_string()),
            raw_markdown: Some("updated_value".to_string()),
            cv_filename: Some("updated_value".to_string()),
            embedding_model: Some("updated_value".to_string()),
            embedded_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            created_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            updated_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
        };

        let result = CandidateProfileService::update(&mut conn, created.id, update_dto, "test_user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_candidateProfile() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = CandidateProfileService::create(&mut conn, dto, "test_user").unwrap();

        // Delete it
        let result = CandidateProfileService::delete(&mut conn, created.id);
        assert!(result.is_ok());

        // Verify it's gone
        let find_result = CandidateProfileService::find_by_id(&mut conn, created.id);
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
            CandidateProfileService::create(&mut conn, dto, "test_user").unwrap();
        }

        let sort_specs: &[&str] = &[
            "userId,asc",
            "userId,desc",
            "fullName,asc",
            "fullName,desc",
            "email,asc",
            "email,desc",
            "location,asc",
            "location,desc",
            "yearsOfExperience,asc",
            "yearsOfExperience,desc",
            "skills,asc",
            "skills,desc",
            "experiences,asc",
            "experiences,desc",
            "preferredRoles,asc",
            "preferredRoles,desc",
            "languages,asc",
            "languages,desc",
            "education,asc",
            "education,desc",
            "certifications,asc",
            "certifications,desc",
            "rawMarkdown,asc",
            "rawMarkdown,desc",
            "cvFilename,asc",
            "cvFilename,desc",
            "embeddingModel,asc",
            "embeddingModel,desc",
            "embeddedAt,asc",
            "embeddedAt,desc",
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
            let result = CandidateProfileService::find_all(&mut conn, &page_request);
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
