//! Extraction structuree d'un profil candidat depuis le markdown d'un CV.
//!
//! Transcription du `extractProfile` de `CvIngestionService` (Spring) : meme prompt, meme
//! structure JSON attendue, et meme tolerance — si le modele renvoie du JSON invalide, on retombe
//! sur un profil vide plutot que de faire echouer l'ingestion.
//!
//! Les champs imbriques (competences, experiences...) restent en `serde_json::Value` : ils sont
//! stockes tels quels dans les colonnes `jsonb`, et cela evite qu'une variation de forme du modele
//! fasse echouer toute l'extraction.

use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::services::AiService;

/// Profil extrait du CV, calque sur le `CandidateProfileDto` Java.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ExtractedProfile {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub location: Option<String>,
    pub years_of_experience: Option<i32>,
    pub skills: Option<serde_json::Value>,
    pub experiences: Option<serde_json::Value>,
    pub preferred_roles: Option<serde_json::Value>,
    pub languages: Option<serde_json::Value>,
    pub education: Option<serde_json::Value>,
    pub certifications: Option<serde_json::Value>,
    pub projects: Option<serde_json::Value>,
}

impl ExtractedProfile {
    /// Noms des competences, pour construire le texte d'embedding (gere la forme structuree
    /// `[{"name":...}]` comme la forme historique `["java", ...]`).
    pub fn skill_names(&self) -> Vec<String> {
        Self::names_from(self.skills.as_ref())
    }

    /// Postes vises, stockes comme une simple liste de chaines.
    pub fn preferred_role_names(&self) -> Vec<String> {
        Self::names_from(self.preferred_roles.as_ref())
    }

    fn names_from(value: Option<&serde_json::Value>) -> Vec<String> {
        let Some(serde_json::Value::Array(items)) = value else {
            return Vec::new();
        };
        items
            .iter()
            .filter_map(|item| match item {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Object(map) => {
                    map.get("name").and_then(|n| n.as_str()).map(str::to_owned)
                }
                _ => None,
            })
            .filter(|s| !s.trim().is_empty())
            .collect()
    }

    /// Serialise un champ imbrique pour la colonne `jsonb` correspondante.
    pub fn json_field(value: Option<&serde_json::Value>) -> Option<String> {
        value.map(|v| v.to_string())
    }
}

const PROMPT_HEADER: &str = r#"Extract the candidate profile from this CV in JSON format.
Return ONLY valid JSON matching this structure:
{
  "fullName": "string",
  "email": "string or null",
  "location": "string or null",
  "yearsOfExperience": number or null,
  "skills": [{"id":"string (slug)","name":"string","level":number (1-5)}],
  "experiences": [{"id":"string (slug)","jobTitle":"string","company":"string","startDate":"string","endDate":"string or null","current":boolean,"description":"string","achievements":["string"]}],
  "preferredRoles": ["string"],
  "languages": [{"id":"string (slug)","name":"string","level":"string"}],
  "education": [{"id":"string (slug)","degree":"string","school":"string","startDate":"string","endDate":"string or null","description":"string"}],
  "certifications": [{"id":"string (slug)","name":"string","issuer":"string","date":"string"}]
}

CV content:
"#;

pub struct CvExtractionService;

impl CvExtractionService {
    /// Demande au modele d'extraire le profil. Renvoie un profil vide si le JSON est illisible,
    /// comme la version Java : mieux vaut un profil incomplet qu'un upload en echec.
    pub async fn extract_profile(markdown: &str) -> Result<ExtractedProfile, AppError> {
        if markdown.trim().is_empty() {
            return Ok(ExtractedProfile::default());
        }
        let prompt = format!("{PROMPT_HEADER}{markdown}");
        let raw = AiService::prompt_with_model(
            &prompt,
            &AiService::default_model(),
            "You are a precise data extraction engine. Reply with JSON only, no commentary.",
        )
        .await?;

        let json = Self::clean_json_response(&raw);
        match serde_json::from_str::<ExtractedProfile>(&json) {
            Ok(profile) => Ok(profile),
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse profile extraction, falling back to empty profile");
                Ok(ExtractedProfile::default())
            }
        }
    }

    /// Retire les delimiteurs de bloc de code que les modeles ajoutent souvent autour du JSON.
    pub fn clean_json_response(raw: &str) -> String {
        let mut s = raw.trim();
        if let Some(stripped) = s.strip_prefix("```json") {
            s = stripped;
        } else if let Some(stripped) = s.strip_prefix("```") {
            s = stripped;
        }
        if let Some(stripped) = s.strip_suffix("```") {
            s = stripped;
        }
        s.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_json_code_fences() {
        assert_eq!(CvExtractionService::clean_json_response("```json\n{\"a\":1}\n```"), "{\"a\":1}");
        assert_eq!(CvExtractionService::clean_json_response("```\n{}\n```"), "{}");
        assert_eq!(CvExtractionService::clean_json_response("  {} "), "{}");
    }

    #[test]
    fn reads_skill_names_in_both_shapes() {
        let structured: ExtractedProfile =
            serde_json::from_str(r#"{"skills":[{"name":"Java","level":5},{"name":"Rust"}]}"#).unwrap();
        assert_eq!(structured.skill_names(), vec!["Java", "Rust"]);

        let legacy: ExtractedProfile = serde_json::from_str(r#"{"skills":["Java","Rust"]}"#).unwrap();
        assert_eq!(legacy.skill_names(), vec!["Java", "Rust"]);
    }

    #[test]
    fn builds_the_same_profile_text_as_the_java_backend() {
        let text = crate::services::EmbeddingService::build_profile_text(
            &["Backend Engineer".to_string()],
            &["Java".to_string(), "Rust".to_string()],
            Some("Senior engineer with 10 years of experience."),
        );
        assert_eq!(
            text,
            "Target Roles: Backend Engineer. Skills: Java, Rust. Background: Senior engineer with 10 years of experience."
        );
    }
}
