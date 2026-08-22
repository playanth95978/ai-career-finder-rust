//! Generation de lettre de motivation, transcription du `CoverLetterService` de l'app Spring.

use crate::dto::parse_string_list;
use crate::errors::AppError;
use crate::models::{CandidateProfile, JobOffer};
use crate::services::ai_service::AiService;

/// Longueur de description d'offre transmise au modele. Les annonces scrapees embarquent souvent
/// des kilo-octets de mentions legales et de boilerplate : au-dela, on paie des jetons pour du
/// bruit qui degrade la lettre au lieu de l'ameliorer.
const MAX_DESCRIPTION_CHARS: usize = 4_000;

const PREAMBLE: &str = "Tu es un expert en recrutement qui redige des lettres de motivation \
sobres et concretes. Tu ecris a la premiere personne, sans formule creuse ni superlatif, en \
t'appuyant uniquement sur les elements fournis. Tu n'inventes jamais une experience, un diplome \
ou une competence absente du profil.";

pub struct CoverLetterService;

impl CoverLetterService {
    /// Redige une lettre pour ce profil et cette offre, dans la langue demandee.
    pub async fn generate(
        profile: &CandidateProfile,
        offer: &JobOffer,
        language: Option<&str>,
    ) -> Result<String, AppError> {
        let prompt = Self::build_prompt(profile, offer, language);
        AiService::prompt_with_model(&prompt, &AiService::default_model(), PREAMBLE).await
    }

    fn build_prompt(
        profile: &CandidateProfile,
        offer: &JobOffer,
        language: Option<&str>,
    ) -> String {
        let language = language
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .unwrap_or("fr");

        let skills = parse_string_list(profile.skills.as_deref());
        let roles = parse_string_list(profile.preferred_roles.as_deref());

        let mut prompt = String::new();
        prompt.push_str(&format!(
            "Redige une lettre de motivation en langue '{language}'.\n\n"
        ));

        prompt.push_str("--- OFFRE ---\n");
        prompt.push_str(&format!("Intitule : {}\n", offer.title));
        if let Some(company) = offer.company.as_deref().filter(|s| !s.trim().is_empty()) {
            prompt.push_str(&format!("Entreprise : {company}\n"));
        }
        if let Some(location) = offer.location.as_deref().filter(|s| !s.trim().is_empty()) {
            prompt.push_str(&format!("Lieu : {location}\n"));
        }
        if let Some(contract) = offer.contract_type.as_deref().filter(|s| !s.trim().is_empty()) {
            prompt.push_str(&format!("Contrat : {contract}\n"));
        }
        if let Some(description) = offer.description.as_deref().filter(|s| !s.trim().is_empty()) {
            prompt.push_str(&format!("Description :\n{}\n", truncate(description, MAX_DESCRIPTION_CHARS)));
        }

        prompt.push_str("\n--- CANDIDAT ---\n");
        if let Some(name) = profile.full_name.as_deref().filter(|s| !s.trim().is_empty()) {
            prompt.push_str(&format!("Nom : {name}\n"));
        }
        if let Some(location) = profile.location.as_deref().filter(|s| !s.trim().is_empty()) {
            prompt.push_str(&format!("Lieu : {location}\n"));
        }
        if let Some(years) = profile.years_of_experience {
            prompt.push_str(&format!("Annees d'experience : {years}\n"));
        }
        if !roles.is_empty() {
            prompt.push_str(&format!("Postes recherches : {}\n", roles.join(", ")));
        }
        if !skills.is_empty() {
            prompt.push_str(&format!("Competences : {}\n", skills.join(", ")));
        }
        if let Some(markdown) = profile.raw_markdown.as_deref().filter(|s| !s.trim().is_empty()) {
            prompt.push_str(&format!("\nCV :\n{}\n", truncate(markdown, MAX_DESCRIPTION_CHARS)));
        }

        prompt.push_str(
            "\n--- CONSIGNES ---\n\
             - 250 a 350 mots, trois ou quatre paragraphes.\n\
             - Relie explicitement les competences du candidat aux besoins de l'offre.\n\
             - Pas de competence ni d'experience absente du profil ci-dessus.\n\
             - Renvoie uniquement le corps de la lettre, sans en-tete ni objet ni commentaire.\n",
        );

        prompt
    }
}

/// Tronque sur une frontiere de caractere (et non d'octet) pour ne pas casser l'UTF-8 des accents.
fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max_chars).collect();
    out.push_str("\n[...]");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> CandidateProfile {
        CandidateProfile {
            id: uuid::Uuid::nil(),
            user_id: "alice".to_string(),
            full_name: Some("Alice Martin".to_string()),
            email: None,
            location: Some("Noumea".to_string()),
            years_of_experience: Some(7),
            skills: Some(r#"["rust","axum"]"#.to_string()),
            experiences: None,
            preferred_roles: Some(r#"["Backend"]"#.to_string()),
            languages: None,
            education: None,
            certifications: None,
            raw_markdown: None,
            cv_filename: None,
            embedding_model: None,
            embedded_at: None,
            created_at: None,
            updated_at: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        }
    }

    fn offer() -> JobOffer {
        JobOffer {
            title: "Developpeur backend".to_string(),
            company: Some("ACME".to_string()),
            description: Some("Nous cherchons un profil Rust.".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn prompt_carries_offer_and_profile_facts() {
        let prompt = CoverLetterService::build_prompt(&profile(), &offer(), Some("fr"));
        assert!(prompt.contains("Developpeur backend"));
        assert!(prompt.contains("ACME"));
        assert!(prompt.contains("Alice Martin"));
        // Les competences sont injectees en clair, pas sous forme de JSON brut.
        assert!(prompt.contains("rust, axum"));
        assert!(!prompt.contains(r#"["rust""#));
    }

    #[test]
    fn prompt_defaults_to_french_when_language_is_unusable() {
        for language in [None, Some(""), Some("   ")] {
            let prompt = CoverLetterService::build_prompt(&profile(), &offer(), language);
            assert!(prompt.contains("langue 'fr'"), "langue={language:?}");
        }
    }

    #[test]
    fn prompt_omits_absent_fields_rather_than_writing_empty_labels() {
        let bare = JobOffer {
            title: "Poste".to_string(),
            ..Default::default()
        };
        let prompt = CoverLetterService::build_prompt(&profile(), &bare, Some("en"));
        assert!(!prompt.contains("Entreprise :"));
        assert!(!prompt.contains("Description :"));
    }

    #[test]
    fn truncate_cuts_on_char_boundaries() {
        // Des accents en fin de coupe : une troncature en octets produirait de l'UTF-8 invalide.
        let text = "éééééééééé";
        let cut = truncate(text, 3);
        assert!(cut.starts_with("ééé"));
        assert!(cut.contains("[...]"));
        // En dessous du plafond, le texte est rendu tel quel.
        assert_eq!(truncate("court", 50), "court");
    }
}
