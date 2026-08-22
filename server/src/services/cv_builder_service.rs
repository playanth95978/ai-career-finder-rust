//! Persistance et versionnage du CV Builder, transcription du `CvResumeService` de l'app Spring.
//!
//! [`save`] est le chemin d'auto-sauvegarde : il upsert le CV editable unique de l'utilisateur
//! sans creer d'historique. [`create_version`] prend un instantane explicite, pour que
//! l'historique reste lisible au lieu d'etre noye par une entree par frappe clavier.

use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::{cv_resume, cv_resume_version};
use crate::db::DbConnection;
use crate::dto::job_copilot_dto::SaveResumeDto;
use crate::errors::AppError;
use crate::models::{CvResume, CvResumeVersion, NewCvResume, NewCvResumeVersion};

pub struct CvBuilderService;

impl CvBuilderService {
    /// CV courant de l'utilisateur, s'il en a un.
    pub fn find_for_user(
        conn: &mut DbConnection,
        user_id: &str,
    ) -> Result<Option<CvResume>, AppError> {
        Ok(cv_resume::table
            .filter(cv_resume::user_id.eq(user_id))
            .order(cv_resume::created_at.desc().nulls_last())
            .select(CvResume::as_select())
            .first(conn)
            .optional()?)
    }

    /// Auto-sauvegarde : cree ou met a jour le CV de l'utilisateur, sans instantane.
    pub fn save(
        conn: &mut DbConnection,
        user_id: &str,
        dto: SaveResumeDto,
    ) -> Result<CvResume, AppError> {
        let now = Utc::now().naive_utc();

        if let Some(existing) = Self::find_for_user(conn, user_id)? {
            return Ok(diesel::update(cv_resume::table.find(existing.id))
                .set((
                    cv_resume::title.eq(dto.title),
                    cv_resume::template.eq(dto.template),
                    cv_resume::data.eq(dto.data),
                    cv_resume::updated_at.eq(now),
                    cv_resume::last_modified_by.eq(user_id.to_string()),
                    cv_resume::last_modified_date.eq(now),
                ))
                .returning(CvResume::as_returning())
                .get_result(conn)?);
        }

        let new_resume = NewCvResume {
            user_id: user_id.to_string(),
            title: dto.title,
            template: dto.template,
            data: dto.data,
            version_number: 1,
            created_at: Some(now),
            updated_at: Some(now),
            created_by: Some(user_id.to_string()),
            created_date: Some(now),
            last_modified_by: Some(user_id.to_string()),
            last_modified_date: Some(now),
        };

        Ok(diesel::insert_into(cv_resume::table)
            .values(&new_resume)
            .returning(CvResume::as_returning())
            .get_result(conn)?)
    }

    /// Fige l'etat courant du CV dans l'historique et incremente son numero de version.
    pub fn create_version(
        conn: &mut DbConnection,
        user_id: &str,
    ) -> Result<CvResumeVersion, AppError> {
        let resume = Self::find_for_user(conn, user_id)?
            .ok_or_else(|| AppError::BadRequest("Aucun CV a versionner".into()))?;

        let now = Utc::now().naive_utc();
        let snapshot = NewCvResumeVersion {
            version_number: resume.version_number,
            title: resume.title.clone(),
            template: resume.template.clone(),
            data: resume.data.clone(),
            created_at: Some(now),
            resume_id: Some(resume.id),
            created_by: Some(user_id.to_string()),
            created_date: Some(now),
            last_modified_by: Some(user_id.to_string()),
            last_modified_date: Some(now),
        };

        // Instantane puis increment dans la meme transaction : si l'increment echouait seul, la
        // prochaine version reutiliserait un numero deja pris et l'historique deviendrait ambigu.
        conn.transaction(|conn| {
            let saved: CvResumeVersion = diesel::insert_into(cv_resume_version::table)
                .values(&snapshot)
                .returning(CvResumeVersion::as_returning())
                .get_result(conn)?;

            diesel::update(cv_resume::table.find(resume.id))
                .set((
                    cv_resume::version_number.eq(resume.version_number + 1),
                    cv_resume::last_modified_by.eq(user_id.to_string()),
                    cv_resume::last_modified_date.eq(now),
                ))
                .execute(conn)?;

            Ok::<CvResumeVersion, diesel::result::Error>(saved)
        })
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    /// Historique du CV, du plus recent au plus ancien. Une liste vide quand il n'y a pas encore
    /// de CV : l'absence d'historique n'est pas une erreur pour le front.
    pub fn list_versions(
        conn: &mut DbConnection,
        user_id: &str,
    ) -> Result<Vec<CvResumeVersion>, AppError> {
        let Some(resume) = Self::find_for_user(conn, user_id)? else {
            return Ok(Vec::new());
        };

        Ok(cv_resume_version::table
            .filter(cv_resume_version::resume_id.eq(resume.id))
            .order(cv_resume_version::version_number.desc())
            .select(CvResumeVersion::as_select())
            .load(conn)?)
    }

    /// Une version precise, verifiee comme appartenant au CV de l'utilisateur : sans ce filtre,
    /// un identifiant devine donnerait acces au CV de quelqu'un d'autre.
    pub fn find_version(
        conn: &mut DbConnection,
        user_id: &str,
        version_id: Uuid,
    ) -> Result<Option<CvResumeVersion>, AppError> {
        let Some(resume) = Self::find_for_user(conn, user_id)? else {
            return Ok(None);
        };

        Ok(cv_resume_version::table
            .filter(cv_resume_version::id.eq(version_id))
            .filter(cv_resume_version::resume_id.eq(resume.id))
            .select(CvResumeVersion::as_select())
            .first(conn)
            .optional()?)
    }

    /// Restaure un instantane dans le CV courant, apres avoir archive l'etat actuel : restaurer
    /// ne doit jamais faire perdre le travail en cours.
    pub fn restore_version(
        conn: &mut DbConnection,
        user_id: &str,
        version_id: Uuid,
    ) -> Result<Option<CvResume>, AppError> {
        let Some(resume) = Self::find_for_user(conn, user_id)? else {
            return Ok(None);
        };
        let Some(target) = Self::find_version(conn, user_id, version_id)? else {
            return Ok(None);
        };

        Self::create_version(conn, user_id)?;

        let now = Utc::now().naive_utc();
        let restored = diesel::update(cv_resume::table.find(resume.id))
            .set((
                cv_resume::title.eq(target.title),
                cv_resume::template.eq(target.template),
                cv_resume::data.eq(target.data),
                cv_resume::updated_at.eq(now),
                cv_resume::last_modified_by.eq(user_id.to_string()),
                cv_resume::last_modified_date.eq(now),
            ))
            .returning(CvResume::as_returning())
            .get_result(conn)?;

        Ok(Some(restored))
    }
}
