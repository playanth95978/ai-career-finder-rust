//! Ingestion d'un CV : OCR -> extraction structuree -> profil candidat -> embedding.
//!
//! Transcription du `CvIngestionService` de l'application Spring. Deux ecarts assumes :
//!  - pas de repli Tika : l'OCR Mistral accepte aussi bien les PDF que les DOCX, on s'appuie
//!    donc uniquement sur lui ;
//!  - pas d'indexation par chunks dans un vector store de documents (il n'existe pas encore ici) ;
//!    seul l'embedding du profil est calcule, comme le fait `CandidateProfileEmbeddingService`.

use chrono::Utc;
use diesel::prelude::*;
use pgvector::Vector;
use uuid::Uuid;

use crate::db::schema::candidate_profile;
use crate::db::DbConnection;
use crate::errors::AppError;
use crate::models::{CandidateProfile, NewCandidateProfile};
use crate::services::cv_extraction_service::{CvExtractionService, ExtractedProfile};
use crate::services::embedding_service::{EmbeddingService, EMBEDDING_DIMENSIONS};
use crate::services::mistral_ocr_service::MistralOcrService;

pub struct CvIngestionService;

impl CvIngestionService {
    /// Ingere un CV pour un utilisateur : renvoie le profil enregistre.
    pub async fn ingest(
        conn: &mut DbConnection,
        bytes: &[u8],
        filename: Option<&str>,
        mime: Option<&str>,
        user_id: &str,
    ) -> Result<CandidateProfile, AppError> {
        let filename = filename.filter(|f| !f.trim().is_empty()).unwrap_or("cv");
        tracing::info!(user = user_id, file = filename, "CV ingestion start");

        // 1. OCR -> markdown
        let ocr = MistralOcrService::process(bytes, Some(filename), mime).await?;
        let markdown = ocr.raw_text;
        if markdown.trim().is_empty() {
            return Err(AppError::BadRequest(
                "No text could be extracted from this document".into(),
            ));
        }

        // 2. Extraction structuree par le LLM
        let extracted = CvExtractionService::extract_profile(&markdown).await?;

        // 3. Persistance (upsert sur user_id, comme la version Java)
        let saved = Self::save_profile(conn, &extracted, &markdown, filename, user_id)?;

        // 4. Embedding — best-effort : un CV enregistre sans vecteur reste exploitable et le
        //    vecteur pourra etre recalcule plus tard, alors qu'echouer ici perdrait tout l'upload.
        if let Err(e) = Self::embed_profile(conn, saved.id, &extracted, &markdown).await {
            tracing::warn!(profile = %saved.id, error = %e, "Profile embedding failed, profile saved without vector");
        }

        Ok(saved)
    }

    /// Cree ou met a jour le profil de l'utilisateur (on garde le plus recent, comme le Java).
    fn save_profile(
        conn: &mut DbConnection,
        extracted: &ExtractedProfile,
        markdown: &str,
        filename: &str,
        user_id: &str,
    ) -> Result<CandidateProfile, AppError> {
        let now = Utc::now().naive_utc();
        let existing: Option<CandidateProfile> = candidate_profile::table
            .filter(candidate_profile::user_id.eq(user_id))
            .order(candidate_profile::created_at.desc())
            .select(CandidateProfile::as_select())
            .first(conn)
            .optional()?;

        if let Some(existing) = existing {
            let updated = diesel::update(candidate_profile::table.find(existing.id))
                .set((
                    candidate_profile::full_name.eq(extracted.full_name.clone()),
                    candidate_profile::email.eq(extracted.email.clone()),
                    candidate_profile::location.eq(extracted.location.clone()),
                    candidate_profile::years_of_experience.eq(extracted.years_of_experience),
                    candidate_profile::skills.eq(ExtractedProfile::json_field(extracted.skills.as_ref())),
                    candidate_profile::experiences.eq(ExtractedProfile::json_field(extracted.experiences.as_ref())),
                    candidate_profile::preferred_roles.eq(ExtractedProfile::json_field(extracted.preferred_roles.as_ref())),
                    candidate_profile::languages.eq(ExtractedProfile::json_field(extracted.languages.as_ref())),
                    candidate_profile::education.eq(ExtractedProfile::json_field(extracted.education.as_ref())),
                    candidate_profile::certifications.eq(ExtractedProfile::json_field(extracted.certifications.as_ref())),
                    candidate_profile::raw_markdown.eq(markdown.to_string()),
                    candidate_profile::cv_filename.eq(filename.to_string()),
                    candidate_profile::updated_at.eq(now),
                    candidate_profile::last_modified_by.eq(user_id.to_string()),
                    candidate_profile::last_modified_date.eq(now),
                ))
                .returning(CandidateProfile::as_returning())
                .get_result(conn)?;
            return Ok(updated);
        }

        let new_profile = NewCandidateProfile {
            user_id: user_id.to_string(),
            full_name: extracted.full_name.clone(),
            email: extracted.email.clone(),
            location: extracted.location.clone(),
            years_of_experience: extracted.years_of_experience,
            skills: ExtractedProfile::json_field(extracted.skills.as_ref()),
            experiences: ExtractedProfile::json_field(extracted.experiences.as_ref()),
            preferred_roles: ExtractedProfile::json_field(extracted.preferred_roles.as_ref()),
            languages: ExtractedProfile::json_field(extracted.languages.as_ref()),
            education: ExtractedProfile::json_field(extracted.education.as_ref()),
            certifications: ExtractedProfile::json_field(extracted.certifications.as_ref()),
            raw_markdown: Some(markdown.to_string()),
            cv_filename: Some(filename.to_string()),
            embedding_model: None,
            embedded_at: None,
            created_at: Some(now),
            updated_at: Some(now),
            created_by: Some(user_id.to_string()),
            created_date: Some(now),
            last_modified_by: Some(user_id.to_string()),
            last_modified_date: Some(now),
        };

        Ok(diesel::insert_into(candidate_profile::table)
            .values(&new_profile)
            .returning(CandidateProfile::as_returning())
            .get_result(conn)?)
    }

    /// Calcule et enregistre le vecteur du profil, a partir du meme texte que la version Java.
    async fn embed_profile(
        conn: &mut DbConnection,
        profile_id: Uuid,
        extracted: &ExtractedProfile,
        markdown: &str,
    ) -> Result<(), AppError> {
        let text = EmbeddingService::build_profile_text(
            &extracted.preferred_role_names(),
            &extracted.skill_names(),
            Some(markdown),
        );
        if text.trim().is_empty() {
            return Err(AppError::BadRequest("Nothing to embed for this profile".into()));
        }

        let vector = EmbeddingService::embed(&text).await?;
        let now = Utc::now().naive_utc();

        diesel::update(candidate_profile::table.find(profile_id))
            .set((
                candidate_profile::embedding.eq(Some(Vector::from(vector))),
                candidate_profile::embedding_model.eq(EmbeddingService::model()),
                candidate_profile::embedded_at.eq(now),
            ))
            .execute(conn)?;

        tracing::info!(profile = %profile_id, dims = EMBEDDING_DIMENSIONS, "Profile embedded");
        Ok(())
    }
}
