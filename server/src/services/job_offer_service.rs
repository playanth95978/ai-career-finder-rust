use chrono::Utc;
use diesel::prelude::*;

use crate::db::connection::DbConnection;
use crate::db::schema::job_offer;
use crate::dto::{CreateJobOfferDto, PageRequest, UpdateJobOfferDto};
use crate::errors::AppError;
use crate::models::{NewJobOffer, UpdateJobOffer, JobOffer};
use uuid::Uuid;

pub struct JobOfferService;

impl JobOfferService {
    /// Find all jobOffers with pagination
    pub fn find_all(
        conn: &mut DbConnection,
        page_request: &PageRequest,
    ) -> Result<(Vec<JobOffer>, i64), AppError> {
        let page = page_request.page.unwrap_or(0);
        let size = page_request.size.unwrap_or(20).min(100);
        let offset = page * size;

        let total: i64 = job_offer::table
            .count()
            .get_result(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Get primary sort parameter (format: "field,direction" e.g., "name,asc" or "cost,desc")
        let (sort_field, sort_dir) = page_request.primary_sort().unwrap_or(("id", "asc"));

        let is_desc = sort_dir.eq_ignore_ascii_case("desc");

        // Dynamic sorting based on field name
        let results = match sort_field {
            "title" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::title.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::title.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "company" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::company.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::company.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "location" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::location.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::location.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "country" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::country.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::country.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "remote" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::remote.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::remote.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "description" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::description.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::description.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "searchText" | "search_text" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::search_text.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::search_text.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "skills" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::skills.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::skills.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "metadata" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::metadata.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::metadata.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "rawPayload" | "raw_payload" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::raw_payload.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::raw_payload.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "contentHash" | "content_hash" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::content_hash.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::content_hash.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "embeddingStatus" | "embedding_status" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::embedding_status.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::embedding_status.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "embeddingModel" | "embedding_model" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::embedding_model.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::embedding_model.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "reindexVersion" | "reindex_version" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::reindex_version.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::reindex_version.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "retryCount" | "retry_count" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::retry_count.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::retry_count.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "indexingError" | "indexing_error" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::indexing_error.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::indexing_error.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "source" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::source.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::source.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "sourceId" | "source_id" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::source_id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::source_id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "applyUrl" | "apply_url" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::apply_url.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::apply_url.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "salaryMin" | "salary_min" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::salary_min.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::salary_min.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "salaryMax" | "salary_max" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::salary_max.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::salary_max.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "salaryCurrency" | "salary_currency" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::salary_currency.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::salary_currency.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "contractType" | "contract_type" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::contract_type.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::contract_type.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "experienceLevel" | "experience_level" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::experience_level.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::experience_level.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "category" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::category.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::category.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "sourceCategory" | "source_category" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::source_category.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::source_category.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "publishedAt" | "published_at" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::published_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::published_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "createdAt" | "created_at" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::created_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::created_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "indexedAt" | "indexed_at" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::indexed_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::indexed_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "updatedAt" | "updated_at" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::updated_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::updated_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "expiresAt" | "expires_at" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::expires_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::expires_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "lastCheckedAt" | "last_checked_at" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::last_checked_at.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::last_checked_at.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "createdDate" | "created_date" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::created_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::created_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            "lastModifiedDate" | "last_modified_date" => {
                if is_desc {
                    job_offer::table
                        .order(job_offer::last_modified_date.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::last_modified_date.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
            _ => {
                // Default: sort by id
                if is_desc {
                    job_offer::table
                        .order(job_offer::id.desc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                } else {
                    job_offer::table
                        .order(job_offer::id.asc())
                        .limit(size)
                        .offset(offset)
                        .select(JobOffer::as_select()).load(conn)
                }
            }
        }
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((results, total))
    }

    /// Find jobOffer by ID
    pub fn find_by_id(conn: &mut DbConnection, id: Uuid) -> Result<JobOffer, AppError> {
        job_offer::table
            .find(id)
            .select(JobOffer::as_select()).first(conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    AppError::NotFound(format!("JobOffer {} not found", id))
                }
                _ => AppError::Internal(e.to_string()),
            })
    }

    /// Create a new jobOffer
    pub fn create(
        conn: &mut DbConnection,
        dto: CreateJobOfferDto,
        created_by: &str,
    ) -> Result<JobOffer, AppError> {
        let now = Utc::now().naive_utc();

        let new_entity = NewJobOffer {
            title: dto.title,
            company: dto.company,
            location: dto.location,
            country: dto.country,
            remote: dto.remote,
            description: dto.description,
            search_text: dto.search_text,
            skills: dto.skills,
            metadata: dto.metadata,
            raw_payload: dto.raw_payload,
            content_hash: dto.content_hash,
            embedding_status: dto.embedding_status,
            embedding_model: dto.embedding_model,
            reindex_version: dto.reindex_version,
            retry_count: dto.retry_count,
            indexing_error: dto.indexing_error,
            source: dto.source,
            source_id: dto.source_id,
            apply_url: dto.apply_url,
            salary_min: dto.salary_min,
            salary_max: dto.salary_max,
            salary_currency: dto.salary_currency,
            contract_type: dto.contract_type,
            experience_level: dto.experience_level,
            category: dto.category,
            source_category: dto.source_category,
            published_at: dto.published_at,
            created_at: dto.created_at,
            indexed_at: dto.indexed_at,
            updated_at: dto.updated_at,
            expires_at: dto.expires_at,
            last_checked_at: dto.last_checked_at,
            created_by: Some(created_by.to_string()),
            created_date: Some(now),
            last_modified_by: Some(created_by.to_string()),
            last_modified_date: Some(now),
        };

        // `returning` explicite : la table porte une colonne `embedding` que le modele
        // `JobOffer` n'expose pas (768 flottants inutiles ici), donc un RETURNING * implicite ne
        // correspondrait plus a la forme attendue.
        let entity = diesel::insert_into(job_offer::table)
            .values(&new_entity)
            .returning(JobOffer::as_returning())
            .get_result::<JobOffer>(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(entity)
    }

    /// Update an existing jobOffer
    pub fn update(
        conn: &mut DbConnection,
        id: Uuid,
        dto: UpdateJobOfferDto,
        modified_by: &str,
    ) -> Result<JobOffer, AppError> {
        let now = Utc::now().naive_utc();

        let update = UpdateJobOffer {
            title: dto.title,
            company: dto.company,
            location: dto.location,
            country: dto.country,
            remote: dto.remote,
            description: dto.description,
            search_text: dto.search_text,
            skills: dto.skills,
            metadata: dto.metadata,
            raw_payload: dto.raw_payload,
            content_hash: dto.content_hash,
            embedding_status: dto.embedding_status,
            embedding_model: dto.embedding_model,
            reindex_version: dto.reindex_version,
            retry_count: dto.retry_count,
            indexing_error: dto.indexing_error,
            source: dto.source,
            source_id: dto.source_id,
            apply_url: dto.apply_url,
            salary_min: dto.salary_min,
            salary_max: dto.salary_max,
            salary_currency: dto.salary_currency,
            contract_type: dto.contract_type,
            experience_level: dto.experience_level,
            category: dto.category,
            source_category: dto.source_category,
            published_at: dto.published_at,
            created_at: dto.created_at,
            indexed_at: dto.indexed_at,
            updated_at: dto.updated_at,
            expires_at: dto.expires_at,
            last_checked_at: dto.last_checked_at,
            last_modified_by: Some(modified_by.to_string()),
            last_modified_date: Some(now),
        };

        diesel::update(job_offer::table.find(id))
            .set(&update)
            .execute(conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Self::find_by_id(conn, id)
    }

    /// Delete a jobOffer
    pub fn delete(conn: &mut DbConnection, id: Uuid) -> Result<(), AppError> {
        diesel::delete(job_offer::table.find(id))
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

    fn create_test_dto_with_suffix(suffix: u32) -> CreateJobOfferDto {
        let _ = suffix; // Suppress unused warning when there are no String fields
        CreateJobOfferDto {
            title: format!("test_value_{}", suffix),
            company: None,
            location: None,
            country: None,
            remote: None,
            description: None,
            search_text: None,
            skills: None,
            metadata: None,
            raw_payload: None,
            content_hash: None,
            embedding_status: None,
            embedding_model: None,
            reindex_version: None,
            retry_count: None,
            indexing_error: None,
            source: None,
            source_id: None,
            apply_url: None,
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            contract_type: None,
            experience_level: None,
            category: None,
            source_category: None,
            published_at: None,
            created_at: None,
            indexed_at: None,
            updated_at: None,
            expires_at: None,
            last_checked_at: None,
        }
    }

    fn create_test_dto() -> CreateJobOfferDto {
        let suffix = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        create_test_dto_with_suffix(suffix)
    }

    #[test]
    fn test_create_jobOffer() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let dto = create_test_dto();
        let result = JobOfferService::create(&mut conn, dto, "test_user");

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
        let created = JobOfferService::create(&mut conn, dto, "test_user").unwrap();

        // Find it by ID
        let result = JobOfferService::find_by_id(&mut conn, created.id);
        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        let result = JobOfferService::find_by_id(&mut conn, Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn test_find_all() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create a few entities with unique values
        for _ in 0..3 {
            let dto = create_test_dto();
            JobOfferService::create(&mut conn, dto, "test_user").unwrap();
        }

        let page_request = PageRequest {
            page: Some(0),
            size: Some(10),
            sort: Vec::new(),
        };

        let result = JobOfferService::find_all(&mut conn, &page_request);
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
            JobOfferService::create(&mut conn, dto, "test_user").unwrap();
        }

        // Request page 0 with size 2
        let page_request = PageRequest {
            page: Some(0),
            size: Some(2),
            sort: Vec::new(),
        };

        let result = JobOfferService::find_all(&mut conn, &page_request);
        assert!(result.is_ok());
        let (items, _total) = result.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_update_jobOffer() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = JobOfferService::create(&mut conn, dto, "test_user").unwrap();

        // Update it
        let update_dto = UpdateJobOfferDto {
            title: Some("updated_value".to_string()),
            company: Some("updated_value".to_string()),
            location: Some("updated_value".to_string()),
            country: Some("updated_value".to_string()),
            remote: Some(true),
            description: Some("updated_value".to_string()),
            search_text: Some("updated_value".to_string()),
            skills: Some("updated_value".to_string()),
            metadata: Some("updated_value".to_string()),
            raw_payload: Some("updated_value".to_string()),
            content_hash: Some("updated_value".to_string()),
            embedding_status: Some("updated_value".to_string()),
            embedding_model: Some("updated_value".to_string()),
            reindex_version: Some(999),
            retry_count: Some(999),
            indexing_error: Some("updated_value".to_string()),
            source: Some("updated_value".to_string()),
            source_id: Some("updated_value".to_string()),
            apply_url: Some("updated_value".to_string()),
            salary_min: Some(999),
            salary_max: Some(999),
            salary_currency: Some("updated_value".to_string()),
            contract_type: Some("updated_value".to_string()),
            experience_level: Some("updated_value".to_string()),
            category: Some("updated_value".to_string()),
            source_category: Some("updated_value".to_string()),
            published_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            created_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            indexed_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            updated_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            expires_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
            last_checked_at: Some(chrono::NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()),
        };

        let result = JobOfferService::update(&mut conn, created.id, update_dto, "test_user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_jobOffer() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();

        // Create an entity
        let dto = create_test_dto();
        let created = JobOfferService::create(&mut conn, dto, "test_user").unwrap();

        // Delete it
        let result = JobOfferService::delete(&mut conn, created.id);
        assert!(result.is_ok());

        // Verify it's gone
        let find_result = JobOfferService::find_by_id(&mut conn, created.id);
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
            JobOfferService::create(&mut conn, dto, "test_user").unwrap();
        }

        let sort_specs: &[&str] = &[
            "title,asc",
            "title,desc",
            "company,asc",
            "company,desc",
            "location,asc",
            "location,desc",
            "country,asc",
            "country,desc",
            "remote,asc",
            "remote,desc",
            "description,asc",
            "description,desc",
            "searchText,asc",
            "searchText,desc",
            "skills,asc",
            "skills,desc",
            "metadata,asc",
            "metadata,desc",
            "rawPayload,asc",
            "rawPayload,desc",
            "contentHash,asc",
            "contentHash,desc",
            "embeddingStatus,asc",
            "embeddingStatus,desc",
            "embeddingModel,asc",
            "embeddingModel,desc",
            "reindexVersion,asc",
            "reindexVersion,desc",
            "retryCount,asc",
            "retryCount,desc",
            "indexingError,asc",
            "indexingError,desc",
            "source,asc",
            "source,desc",
            "sourceId,asc",
            "sourceId,desc",
            "applyUrl,asc",
            "applyUrl,desc",
            "salaryMin,asc",
            "salaryMin,desc",
            "salaryMax,asc",
            "salaryMax,desc",
            "salaryCurrency,asc",
            "salaryCurrency,desc",
            "contractType,asc",
            "contractType,desc",
            "experienceLevel,asc",
            "experienceLevel,desc",
            "category,asc",
            "category,desc",
            "sourceCategory,asc",
            "sourceCategory,desc",
            "publishedAt,asc",
            "publishedAt,desc",
            "createdAt,asc",
            "createdAt,desc",
            "indexedAt,asc",
            "indexedAt,desc",
            "updatedAt,asc",
            "updatedAt,desc",
            "expiresAt,asc",
            "expiresAt,desc",
            "lastCheckedAt,asc",
            "lastCheckedAt,desc",
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
            let result = JobOfferService::find_all(&mut conn, &page_request);
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
