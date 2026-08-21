use async_trait::async_trait;
use chrono::NaiveDateTime;
use reqwest::Client;
use serde::Deserialize;

use crate::errors::AppError;
use crate::models::JobOffer;
use super::ats_connector::{AtsConnector, JobSource};

const RECORDS_API: &str =
    "https://data.gouv.nc/api/explore/v2.1/catalog/datasets/offres-d-emploi-deposees-sur-le-site-emploi-nc/records";

const OFFER_URL_PREFIX: &str = "https://emploi.nc/offers/";

const PAGE_SIZE: usize = 50;

/// Connector for New Caledonia job offers published on emploi.nc.
pub struct EmploiNcConnector {
    client: Client,
}

impl EmploiNcConnector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    // -------------------------------------------------------------------------
    // HTTP
    // -------------------------------------------------------------------------

    async fn fetch_page(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
        offset: usize,
    ) -> Result<Vec<JobOffer>, AppError> {
        let where_clause = self.build_where(keywords, location);
        let limit_str = PAGE_SIZE.to_string();
        let offset_str = offset.to_string();

        let query_params = [
            ("where", where_clause.as_str()),
            ("limit", limit_str.as_str()),
            ("offset", offset_str.as_str()),
            ("order_by", "date_debut desc"),
        ];

        let records = self
            .client
            .get(RECORDS_API)
            .query(&query_params)
            .send()
            .await?
            .error_for_status()?
            .json::<EmploiNcRecords>()
            .await?;

        Ok(records
            .results
            .unwrap_or_default()
            .into_iter()
            .map(|record| self.map_job_offer(record))
            .collect())
    }

    // -------------------------------------------------------------------------
    // Query building
    // -------------------------------------------------------------------------

    /// Builds an ODSQL `where` clause filtering on active offers,
    /// keywords and city.
    fn build_where(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> String {
        let mut where_clause = String::from(r#"statut="ACTIVE""#);

        if let Some(keywords) = keywords.filter(|value| !value.trim().is_empty()) {
            where_clause.push_str(&format!(
                r#" and search("{}")"#,
                self.sanitize(keywords)
            ));
        }

        if let Some(location) = location.filter(|value| !value.trim().is_empty()) {
            where_clause.push_str(&format!(
                r#" and search(ville, "{}")"#,
                self.sanitize(location)
            ));
        }

        where_clause
    }

    /// Removes characters that would break the ODSQL string literal.
    fn sanitize(&self, value: &str) -> String {
        value.replace('"', " ").trim().to_string()
    }

    // -------------------------------------------------------------------------
    // Mapping
    // -------------------------------------------------------------------------

    fn map_job_offer(&self, record: EmploiNcOffer) -> JobOffer {
        let uuid = trim_to_none(record.uuid.as_deref());

        let company = first_non_blank(
            record.enseigne.as_deref(),
            record.designation.as_deref(),
        );

        let location = first_non_blank(
            record.ville.as_deref(),
            record.ville_physique.as_deref(),
        );

        let apply_url = match &uuid {
            Some(uuid) => format!("{OFFER_URL_PREFIX}{uuid}#top"),
            None => OFFER_URL_PREFIX.to_string(),
        };

        JobOffer {
            id: uuid::Uuid::new_v4(),
            title: trim_to_none(record.titre.as_deref()).unwrap_or_else(|| "Sans titre".to_string()),
            company,
            location,
            country: Some("Nouvelle-Calédonie".to_string()),
            contract_type: trim_to_none(record.type_contrat.as_deref()),

            apply_url: Some(apply_url),
            source_id: uuid,

            source: Some(JobSource::EmploiNc.as_str().to_string()),

            remote: Some(false),

            published_at: record
                .date_debut
                .as_deref()
                .and_then(parse_date),

            description: Some(self.build_description(&record)),

            ..Default::default()
        }
    }

    /// Builds a non-empty description.
    fn build_description(&self, record: &EmploiNcOffer) -> String {
        if let Some(description) = record
            .description
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            return description.to_string();
        }

        let title = record
            .titre
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("emploi");

        let mut description = format!("Offre {title}");

        if let Some(contract_type) = record
            .type_contrat
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            description.push_str(&format!(" — {contract_type}"));
        }

        let city = first_non_blank(
            record.ville.as_deref(),
            record.ville_physique.as_deref(),
        );

        if let Some(city) = city {
            description.push_str(&format!(" à {city}"));
        }

        description.push('.');

        if let Some(email) = record
            .email
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            description.push_str(&format!(" Contact : {email}."));
        }

        if let Some(phone) = record
            .telephone
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            description.push_str(&format!(" Tél : {phone}."));
        }

        description.push_str(" Source : emploi.nc.");

        description
    }
}

// =============================================================================
// AtsConnector implementation
// =============================================================================

#[async_trait]
impl AtsConnector for EmploiNcConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("emploi.nc") || url.contains("emploi.gouv.nc")
    }

    fn source(&self) -> JobSource {
        JobSource::EmploiNc
    }

    async fn fetch_jobs(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        self.fetch_page(keywords, location, 0).await
    }

    async fn fetch_from_url(
        &self,
        _url: &str,
    ) -> Result<Vec<JobOffer>, AppError> {
        // A bare emploi.nc URL maps to the active-offers feed.
        self.fetch_jobs(None, None).await
    }
}

// =============================================================================
// API payload
// =============================================================================

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EmploiNcRecords {
    #[serde(default)]
    results: Option<Vec<EmploiNcOffer>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EmploiNcOffer {
    uuid: Option<String>,
    titre: Option<String>,
    enseigne: Option<String>,
    designation: Option<String>,
    ville: Option<String>,
    ville_physique: Option<String>,
    type_contrat: Option<String>,
    description: Option<String>,
    email: Option<String>,
    telephone: Option<String>,
    date_debut: Option<String>,
    statut: Option<String>,
}

// =============================================================================
// String utilities
// =============================================================================

fn trim_to_none(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
}

fn first_non_blank(first: Option<&str>, second: Option<&str>) -> Option<String> {
    first
        .filter(|value| !value.trim().is_empty())
        .or_else(|| second.filter(|value| !value.trim().is_empty()))
        .map(|value| value.trim().to_string())
}

fn parse_date(date_str: &str) -> Option<NaiveDateTime> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        return Some(dt.naive_utc());
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
        return Some(ndt);
    }
    if let Ok(nd) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return nd.and_hms_opt(0, 0, 0);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports() {
        let connector = EmploiNcConnector::new(Client::new());
        assert!(connector.supports("https://emploi.nc/offers/123"));
        assert!(connector.supports("https://emploi.gouv.nc/"));
        assert!(!connector.supports("https://example.com/"));
    }

    #[test]
    fn test_build_where() {
        let connector = EmploiNcConnector::new(Client::new());
        assert_eq!(connector.build_where(None, None), r#"statut="ACTIVE""#);
        assert_eq!(
            connector.build_where(Some("developer"), Some("Nouméa")),
            r#"statut="ACTIVE" and search("developer") and search(ville, "Nouméa")"#
        );
    }

    #[test]
    fn test_parse_date() {
        assert!(parse_date("2024-01-15").is_some());
        assert!(parse_date("2024-01-15T10:30:00").is_some());
        assert!(parse_date("2024-01-15T10:30:00Z").is_some());
        assert!(parse_date("invalid-date").is_none());
    }

    #[test]
    fn test_map_job_offer() {
        let connector = EmploiNcConnector::new(Client::new());
        let offer = EmploiNcOffer {
            uuid: Some("abc-123".to_string()),
            titre: Some("Développeur Rust".to_string()),
            enseigne: Some("OPT-NC".to_string()),
            designation: None,
            ville: Some("Nouméa".to_string()),
            ville_physique: None,
            type_contrat: Some("CDI".to_string()),
            description: None,
            email: Some("recrutement@opt.nc".to_string()),
            telephone: None,
            date_debut: Some("2024-01-15".to_string()),
            statut: Some("ACTIVE".to_string()),
        };

        let mapped = connector.map_job_offer(offer);
        assert_eq!(mapped.title, "Développeur Rust");
        assert_eq!(mapped.company, Some("OPT-NC".to_string()));
        assert_eq!(mapped.location, Some("Nouméa".to_string()));
        assert_eq!(mapped.contract_type, Some("CDI".to_string()));
        assert_eq!(mapped.source, Some("EMPLOI_NC".to_string()));
        assert_eq!(mapped.source_id, Some("abc-123".to_string()));
        assert_eq!(mapped.apply_url, Some("https://emploi.nc/offers/abc-123#top".to_string()));
        assert!(mapped.published_at.is_some());
        assert!(mapped.description.unwrap().contains("Offre Développeur Rust"));
    }
}