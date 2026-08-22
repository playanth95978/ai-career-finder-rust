//! DTO propres aux endpoints metier du Job Copilot (`/api/job-copilot/**`, `/api/cv-builder/**`).
//!
//! Ils ne derivent pas des entites : le front attend les formes definies dans
//! `job-copilot.model.ts` (tableaux plutot que colonnes TEXT contenant du JSON, tags de match
//! traduisibles, etc.). Les DTO d'entite generes par JHipster restent utilises par le CRUD REST.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{CvResume, CvResumeVersion, JobOffer};

// ---------------------------------------------------------------------------
// Offres, match
// ---------------------------------------------------------------------------

/// Vue allegee d'une offre, telle que l'attend `JobOfferDto` cote Angular : `skills` est un
/// tableau (la colonne stocke du JSON) et `remote` n'est jamais nul.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobOfferSummaryDto {
    pub id: Uuid,
    pub title: String,
    pub company: Option<String>,
    pub location: Option<String>,
    pub remote: bool,
    pub description: Option<String>,
    pub skills: Vec<String>,
    pub source: Option<String>,
    pub apply_url: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub contract_type: Option<String>,
}

impl From<&JobOffer> for JobOfferSummaryDto {
    fn from(offer: &JobOffer) -> Self {
        Self {
            id: offer.id,
            title: offer.title.clone(),
            company: offer.company.clone(),
            location: offer.location.clone(),
            remote: offer.remote.unwrap_or(false),
            description: offer.description.clone(),
            skills: parse_string_list(offer.skills.as_deref()),
            source: offer.source.clone(),
            apply_url: offer.apply_url.clone(),
            salary_min: offer.salary_min,
            salary_max: offer.salary_max,
            contract_type: offer.contract_type.clone(),
        }
    }
}

/// Un point fort ou faible du match, sous forme de cle i18n resolue par le front
/// (`jobCopilot.match.tag.*`) — jamais de texte en dur cote serveur.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MatchTagDto {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
}

impl MatchTagDto {
    pub fn of(key: &str) -> Self {
        Self { key: key.to_string(), skill: None }
    }

    pub fn with_skill(key: &str, skill: &str) -> Self {
        Self { key: key.to_string(), skill: Some(skill.to_string()) }
    }
}

/// Catalogue des cles de tags produites par le scoring deterministe.
/// Miroir de l'enum `MatchTag` cote Java : les bundles i18n du front sont indexes sur ces cles.
pub mod match_tag {
    pub const MATCHED_SKILL: &str = "jobCopilot.match.tag.matchedSkill";
    pub const MISSING_SKILL: &str = "jobCopilot.match.tag.missingSkill";
    pub const LOCATION_MATCH: &str = "jobCopilot.match.tag.locationMatch";
    pub const PREFERRED_LOCATION: &str = "jobCopilot.match.tag.preferredLocation";
    pub const LOCATION_MISMATCH: &str = "jobCopilot.match.tag.locationMismatch";
    pub const REMOTE: &str = "jobCopilot.match.tag.remote";
    pub const NOT_REMOTE: &str = "jobCopilot.match.tag.notRemote";
    pub const SALARY_COMPATIBLE: &str = "jobCopilot.match.tag.salaryCompatible";
    pub const SALARY_BELOW: &str = "jobCopilot.match.tag.salaryBelow";
    pub const STRONG_MATCH: &str = "jobCopilot.match.tag.strongMatch";
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MatchResultDto {
    pub job_offer: JobOfferSummaryDto,
    pub score: f64,
    pub strengths: Vec<MatchTagDto>,
    pub weaknesses: Vec<MatchTagDto>,
}

/// Valeurs de filtre disponibles sous les criteres courants, avec leur cardinalite. Le client ne
/// detient qu'une page de resultats : il ne peut rien compter lui-meme.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OfferFacetsDto {
    pub locations: Vec<FacetValueDto>,
    pub companies: Vec<FacetValueDto>,
    pub contract_types: Vec<FacetValueDto>,
    pub categories: Vec<FacetValueDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FacetValueDto {
    pub value: String,
    pub count: i64,
}

/// Reponse `{ "count": n }`, forme attendue par `indexed-count` et `radar/unread-count`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CountDto {
    pub count: i64,
}

// ---------------------------------------------------------------------------
// Radar
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RadarHitFeedDto {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_offer: Option<JobOfferSummaryDto>,
    pub score: f64,
    pub why_you: Vec<MatchTagDto>,
    pub seen: bool,
    pub dismissed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Candidatures
// ---------------------------------------------------------------------------

/// Candidature telle que l'affiche le suivi cote front.
///
/// Le DTO d'entite genere (`JobApplicationDto`) n'imbrique de l'offre que son `id` et son
/// `title` ; l'ecran de suivi affiche aussi l'entreprise, le lieu et l'URL de candidature. On
/// expose donc l'offre complete via [`JobOfferSummaryDto`] plutot que de laisser le front faire
/// une requete par ligne.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobApplicationViewDto {
    pub id: Uuid,
    pub user_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_letter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_offer: Option<JobOfferSummaryDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_at: Option<String>,
}

impl JobApplicationViewDto {
    /// `status` est non-nul cote front (union de chaines) : une colonne NULL en base ressort en
    /// `DRAFT` plutot que d'etre omise, ce qui casserait le rendu du tableau.
    pub fn new(
        application: &crate::models::JobApplication,
        offer: Option<&JobOffer>,
        default_status: &str,
    ) -> Self {
        Self {
            id: application.id,
            user_id: application.user_id.clone(),
            status: application
                .status
                .clone()
                .unwrap_or_else(|| default_status.to_string()),
            cover_letter: application.cover_letter.clone(),
            notes: application.notes.clone(),
            match_score: application.match_score,
            job_offer: offer.map(JobOfferSummaryDto::from),
            created_at: application.created_at.map(|d| d.to_string()),
            updated_at: application.updated_at.map(|d| d.to_string()),
            applied_at: application.applied_at.map(|d| d.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Preferences
// ---------------------------------------------------------------------------

/// Corps de `PUT /api/job-copilot/preferences`. Les trois listes arrivent en tableaux et sont
/// stockees en JSON dans des colonnes TEXT.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferenceInputDto {
    pub remote_only: Option<bool>,
    pub contract_type: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    #[serde(default)]
    pub preferred_roles: Vec<String>,
    #[serde(default)]
    pub excluded_technologies: Vec<String>,
    #[serde(default)]
    pub preferred_locations: Vec<String>,
}

// ---------------------------------------------------------------------------
// CV Builder
// ---------------------------------------------------------------------------

/// CV echange avec le CV Builder. `data` est le JSON complet du CV, opaque pour le backend ;
/// `title`/`template` sont denormalises pour lister sans parser.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResumeDto {
    pub id: Uuid,
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: String,
    pub version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl From<CvResume> for ResumeDto {
    fn from(r: CvResume) -> Self {
        Self {
            id: r.id,
            title: r.title,
            template: r.template,
            data: r.data,
            version: r.version_number,
            updated_at: r.updated_at.map(|d| d.to_string()),
        }
    }
}

/// Corps de `PUT /api/cv-builder/resume`. `id`/`version` sont ignores : le serveur upsert le CV
/// unique de l'utilisateur et gere lui-meme la numerotation.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SaveResumeDto {
    pub title: Option<String>,
    pub template: Option<String>,
    pub data: String,
}

/// Entree d'historique, sans `data` : la liste des versions n'a pas besoin du contenu.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResumeVersionDto {
    pub id: Uuid,
    pub version: i32,
    pub title: Option<String>,
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl From<CvResumeVersion> for ResumeVersionDto {
    fn from(v: CvResumeVersion) -> Self {
        Self {
            id: v.id,
            version: v.version_number,
            title: v.title,
            template: v.template,
            created_at: v.created_at.map(|d| d.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers de parsing JSON-en-TEXT
// ---------------------------------------------------------------------------

/// Lit une colonne TEXT contenant une liste JSON.
///
/// Deux formats coexistent en base : la liste de chaines historique (`["rust","axum"]`) et la
/// liste structuree produite par l'extraction de CV (`[{"name":"rust",...}]`). Les deux sont
/// acceptes ; un JSON illisible est traite comme une liste vide plutot que de faire echouer la
/// reponse entiere.
pub fn parse_string_list(raw: Option<&str>) -> Vec<String> {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Vec::new();
    };

    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };

    let Some(items) = value.as_array() else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| match item {
            serde_json::Value::String(s) => Some(s.clone()),
            // Forme structuree : on ne garde que le nom, seul champ commun aux deux formats.
            serde_json::Value::Object(map) => map
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            _ => None,
        })
        .filter(|s| !s.trim().is_empty())
        .collect()
}

/// Serialise une liste vers une colonne TEXT. `None` quand la liste est vide, pour ne pas
/// distinguer « jamais renseigne » de « renseigne vide » en base.
pub fn to_json_text(values: &[String]) -> Option<String> {
    if values.is_empty() {
        return None;
    }
    serde_json::to_string(values).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_string_list_reads_plain_string_arrays() {
        assert_eq!(
            parse_string_list(Some(r#"["rust","axum"]"#)),
            vec!["rust".to_string(), "axum".to_string()]
        );
    }

    #[test]
    fn parse_string_list_reads_structured_skills() {
        assert_eq!(
            parse_string_list(Some(r#"[{"name":"rust","level":"expert"},{"name":"axum"}]"#)),
            vec!["rust".to_string(), "axum".to_string()]
        );
    }

    #[test]
    fn parse_string_list_treats_unusable_input_as_empty() {
        assert!(parse_string_list(None).is_empty());
        assert!(parse_string_list(Some("   ")).is_empty());
        assert!(parse_string_list(Some("not json")).is_empty());
        // Un objet n'est pas une liste : pas de valeur exploitable.
        assert!(parse_string_list(Some(r#"{"a":1}"#)).is_empty());
        // Elements sans `name` ni chaine : ignores un a un, pas d'echec global.
        assert_eq!(parse_string_list(Some(r#"["ok",42,{"x":1}]"#)), vec!["ok".to_string()]);
    }

    #[test]
    fn to_json_text_maps_empty_to_none() {
        assert_eq!(to_json_text(&[]), None);
        assert_eq!(
            to_json_text(&["rust".to_string()]).as_deref(),
            Some(r#"["rust"]"#)
        );
    }

    #[test]
    fn match_tag_dto_carries_optional_skill() {
        let plain = MatchTagDto::of(match_tag::REMOTE);
        assert_eq!(plain.key, "jobCopilot.match.tag.remote");
        assert!(plain.skill.is_none());
        // La cle d'interpolation `{{skill}}` du front n'a de sens que sur les tags de competence.
        let skilled = MatchTagDto::with_skill(match_tag::MATCHED_SKILL, "rust");
        assert_eq!(skilled.skill.as_deref(), Some("rust"));
    }
}
