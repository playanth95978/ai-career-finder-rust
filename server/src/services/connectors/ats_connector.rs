use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::models::JobOffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobSource {
    Greenhouse,
    Lever,
    SmartRecruiters,
    Ashby,
    Workable,
    Recruitee,
    RemoteOk,
    Jobicy,
    Adzuna,
    Careerjet,
    WeWorkRemotely,
    FranceTravail,
    EmploiNc,
    HelloWork,
    Seek,
    Manual,
    LinkedIn,
    WelcomeToTheJungle,
}

impl JobSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobSource::Greenhouse => "GREENHOUSE",
            JobSource::Lever => "LEVER",
            JobSource::SmartRecruiters => "SMARTRECRUITERS",
            JobSource::Ashby => "ASHBY",
            JobSource::Workable => "WORKABLE",
            JobSource::Recruitee => "RECRUITEE",
            JobSource::RemoteOk => "REMOTEOK",
            JobSource::Jobicy => "JOBICY",
            JobSource::Adzuna => "ADZUNA",
            JobSource::Careerjet => "CAREERJET",
            JobSource::WeWorkRemotely => "WEWORKREMOTELY",
            JobSource::FranceTravail => "FRANCE_TRAVAIL",
            JobSource::EmploiNc => "EMPLOI_NC",
            JobSource::HelloWork => "HELLOWORK",
            JobSource::Seek => "SEEK",
            JobSource::Manual => "MANUAL",
            JobSource::LinkedIn => "LINKEDIN",
            JobSource::WelcomeToTheJungle => "WELCOME_TO_THE_JUNGLE",
        }
    }
}

impl std::fmt::Display for JobSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Send + Sync : les connecteurs sont partages entre taches Tokio et manipules en `dyn`.
#[async_trait]
pub trait AtsConnector: Send + Sync {
    /// Returns true if this connector supports the given URL.
    fn supports(&self, url: &str) -> bool;

    /// Returns the source associated with this connector.
    fn source(&self) -> JobSource;

    /// Fetch jobs using keywords and location.
    async fn fetch_jobs(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError>;

    /// Country-aware fetch.
    ///
    /// Only connectors whose API is partitioned by country use `country`.
    /// Other connectors ignore it and fall back to the two-argument form.
    async fn fetch_jobs_by_country(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
        _country: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        self.fetch_jobs(keywords, location).await
    }

    /// Fetches jobs with pagination.
    ///
    /// Default implementation simply returns all jobs as a stream.
    async fn fetch_jobs_paginated(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        self.fetch_jobs(keywords, location).await
    }

    /// Fetches the full details of a job offer.
    ///
    /// Returns `None` when this connector cannot enrich the URL.
    async fn fetch_detail(
        &self,
        _url: &str,
    ) -> Result<Option<JobOffer>, AppError> {
        Ok(None)
    }

    /// Fetches every posting for a company board identified by its slug.
    ///
    /// Default: no results.
    async fn fetch_by_slug(
        &self,
        _slug: &str,
    ) -> Result<Vec<JobOffer>, AppError> {
        Ok(Vec::new())
    }

    /// Fetches offers from a supported board/feed URL.
    ///
    /// Default: no results.
    async fn fetch_from_url(
        &self,
        _url: &str,
    ) -> Result<Vec<JobOffer>, AppError> {
        Ok(Vec::new())
    }

    /// Parses an ATS-specific posting reference.
    ///
    /// Default: this connector cannot parse the URL.
    fn parse_ref(&self, _url: &str) -> Option<PostingRef> {
        None
    }
}

/// Extracts the path segment immediately after `marker`.
///
/// Example:
/// board_slug("https://jobs.lever.co/acme/job/123", "jobs.lever.co/")
/// => "acme"
pub fn board_slug(url: Option<&str>, marker: &str) -> String {
    let Some(url) = url else {
        return String::new();
    };

    let Some(idx) = url.find(marker) else {
        return url.to_string();
    };

    let rest = &url[idx + marker.len()..];

    match rest.find('/') {
        Some(slash) if slash > 0 => rest[..slash].to_string(),
        _ => rest.to_string(),
    }
}

/// Parsing de dates tolerant, partage par les connecteurs.
///
/// Les sources d'offres ne sont pas homogenes : l'opendata emploi.nc renvoie tantot une date seule
/// (`2026-05-01`), tantot un horodatage ISO avec fuseau. On essaie donc les formats du plus precis
/// au moins precis, et on renvoie `None` plutot que d'echouer : une date de publication absente
/// n'est pas une raison de rejeter une offre.
pub struct AtsDates;

impl AtsDates {
    pub fn parse(raw: &str) -> Option<chrono::NaiveDateTime> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        // 1. ISO 8601 complet avec fuseau -> ramene en UTC naif.
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(raw) {
            return Some(dt.naive_utc());
        }
        // 2. Horodatage sans fuseau, avec ou sans fraction de seconde.
        for fmt in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f"] {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(raw, fmt) {
                return Some(dt);
            }
        }
        // 3. Date seule -> minuit.
        chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
    }
}

/// A parsed reference to a specific posting on an ATS.
#[derive(Debug, Clone)]
pub struct PostingRef {
    pub site: String,
    pub posting_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_slug() {
        assert_eq!(
            board_slug(Some("https://jobs.lever.co/acme/job/123"), "jobs.lever.co/"),
            "acme"
        );
        assert_eq!(board_slug(None, "jobs.lever.co/"), "");
        assert_eq!(
            board_slug(Some("https://example.com/"), "jobs.lever.co/"),
            "https://example.com/"
        );
    }

    #[test]
    fn test_ats_dates_parse() {
        assert!(AtsDates::parse("2026-05-01").is_some());
        assert!(AtsDates::parse("2026-05-01T10:30:00").is_some());
        assert!(AtsDates::parse("2026-05-01T10:30:00+11:00").is_some());
        assert!(AtsDates::parse("").is_none());
        assert!(AtsDates::parse("pas une date").is_none());
    }

    #[test]
    fn test_job_source_display() {
        assert_eq!(JobSource::Greenhouse.as_str(), "GREENHOUSE");
        assert_eq!(format!("{}", JobSource::Lever), "LEVER");
    }
}