//! Embeddings via Ollama, avec le meme modele que l'application Spring (`nomic-embed-text`, 768
//! dimensions) afin que les vecteurs restent comparables entre les deux backends.

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

const DEFAULT_BASE_URL: &str = "http://localhost:11434";
const DEFAULT_MODEL: &str = "nomic-embed-text";
/// Dimension attendue — doit correspondre a la colonne `vector(768)`.
pub const EMBEDDING_DIMENSIONS: usize = 768;
/// Longueur de CV reprise dans le texte de profil, alignee sur la version Java.
const BACKGROUND_MAX_CHARS: usize = 1000;

#[derive(Debug, Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    #[serde(default)]
    embeddings: Vec<Vec<f32>>,
}

pub struct EmbeddingService;

impl EmbeddingService {
    pub fn model() -> String {
        std::env::var("OLLAMA_EMBEDDING_MODEL")
            .ok()
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string())
    }

    fn base_url() -> String {
        std::env::var("OLLAMA_BASE_URL")
            .ok()
            .filter(|u| !u.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
    }

    /// Calcule l'embedding d'un texte. Erreur si le texte est vide ou si la dimension renvoyee
    /// ne correspond pas a celle de la colonne vectorielle (mauvais modele configure).
    pub async fn embed(text: &str) -> Result<Vec<f32>, AppError> {
        if text.trim().is_empty() {
            return Err(AppError::BadRequest("Cannot embed an empty text".into()));
        }
        let model = Self::model();
        let url = format!("{}/api/embed", Self::base_url().trim_end_matches('/'));

        let response = reqwest::Client::new()
            .post(&url)
            .json(&EmbedRequest { model: &model, input: text })
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Ollama embedding request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Ollama embedding returned {status}: {body}")));
        }

        let parsed: EmbedResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Ollama embedding response parsing failed: {e}")))?;

        let vector = parsed
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Ollama returned no embedding".into()))?;

        if vector.len() != EMBEDDING_DIMENSIONS {
            return Err(AppError::Internal(format!(
                "Embedding model '{model}' returned {} dimensions, expected {EMBEDDING_DIMENSIONS}",
                vector.len()
            )));
        }
        Ok(vector)
    }

    /// Rendu textuel canonique d'un profil, transcrit du `buildProfileText` Java : postes vises,
    /// competences, puis un extrait du CV brut. Les deux backends embarquent ainsi le meme texte.
    pub fn build_profile_text(
        preferred_roles: &[String],
        skill_names: &[String],
        raw_markdown: Option<&str>,
    ) -> String {
        let mut out = String::new();
        if !preferred_roles.is_empty() {
            out.push_str(&format!("Target Roles: {}. ", preferred_roles.join(", ")));
        }
        if !skill_names.is_empty() {
            out.push_str(&format!("Skills: {}. ", skill_names.join(", ")));
        }
        if let Some(md) = raw_markdown.map(str::trim).filter(|m| !m.is_empty()) {
            // Troncature sur une frontiere de caractere (le slice d'octets paniquerait en UTF-8).
            let background: String = md.chars().take(BACKGROUND_MAX_CHARS).collect();
            out.push_str("Background: ");
            out.push_str(&background);
        }
        out.trim().to_string()
    }
}
