//! OCR de documents via l'API Mistral (`POST /v1/ocr`).
//!
//! Portage du `MistralOCRProcessService` Spring : le document est envoyé *inline* en data-URI
//! base64 (aucun upload préalable), le modèle `mistral-ocr-latest` renvoie une page par page
//! de markdown, et l'on concatène ces pages pour obtenir le texte exploitable.
//!
//! Différence assumée avec la version Java : l'indexation vectorielle (embedding + chunking
//! markdown) n'est pas reprise ici — il n'y a pas encore de vector store côté Rust. `process`
//! renvoie donc directement le markdown, à charge de l'appelant de l'indexer plus tard.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;

const OCR_URL: &str = "https://api.mistral.ai/v1/ocr";
const OCR_MODEL: &str = "mistral-ocr-latest";
const DEFAULT_MIME: &str = "application/pdf";
const DEFAULT_FILENAME: &str = "document";

/// Filigrane laissé par certains générateurs de CV, retiré comme dans la version Java.
const WATERMARK: &str = "www.enhancv.com";

#[derive(Debug, Serialize)]
struct OcrRequest<'a> {
    model: &'a str,
    /// Nom logique du document — repris tel quel par Mistral dans sa réponse.
    id: &'a str,
    document: DocumentChunk,
    include_image_base64: bool,
}

#[derive(Debug, Serialize)]
struct DocumentChunk {
    #[serde(rename = "type")]
    kind: &'static str,
    document_url: String,
}

#[derive(Debug, Deserialize)]
pub struct OcrResponse {
    #[serde(default)]
    pub pages: Vec<OcrPage>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OcrPage {
    #[serde(default)]
    pub index: i32,
    #[serde(default)]
    pub markdown: String,
}

/// Résultat rendu à l'appelant : identifiant de document + markdown extrait.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrResult {
    pub doc_id: String,
    pub filename: String,
    pub raw_text: String,
}

pub struct MistralOcrService;

impl MistralOcrService {
    /// Point d'entrée : OCR du document puis extraction du markdown.
    pub async fn process(bytes: &[u8], filename: Option<&str>, mime: Option<&str>) -> Result<OcrResult, AppError> {
        let filename = filename.filter(|f| !f.trim().is_empty()).unwrap_or(DEFAULT_FILENAME);
        let response = Self::perform_ocr(bytes, Some(filename), mime).await?;
        Ok(OcrResult {
            doc_id: Uuid::new_v4().to_string(),
            filename: filename.to_owned(),
            raw_text: Self::extract_markdown(&response),
        })
    }

    /// Appelle l'API OCR. Le document part en data-URI base64, comme dans la version Java.
    pub async fn perform_ocr(bytes: &[u8], filename: Option<&str>, mime: Option<&str>) -> Result<OcrResponse, AppError> {
        let api_key = std::env::var("MISTRAL_API_KEY")
            .map_err(|_| AppError::Internal("MISTRAL_API_KEY is not set".into()))?;

        let filename = filename.filter(|f| !f.trim().is_empty()).unwrap_or(DEFAULT_FILENAME);
        let mime = mime.filter(|m| !m.trim().is_empty()).unwrap_or(DEFAULT_MIME);
        let data_uri = format!("data:{};base64,{}", mime, BASE64.encode(bytes));

        tracing::info!(file = filename, bytes = bytes.len(), mime, "OCR start");

        let request = OcrRequest {
            model: OCR_MODEL,
            id: filename,
            document: DocumentChunk { kind: "document_url", document_url: data_uri },
            include_image_base64: true,
        };

        let response = reqwest::Client::new()
            .post(OCR_URL)
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Mistral OCR request failed: {e}")))?;

        // Le corps d'erreur de Mistral est informatif : on le remonte plutôt que le seul statut.
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Mistral OCR returned {status}: {body}")));
        }

        response
            .json::<OcrResponse>()
            .await
            .map_err(|e| AppError::Internal(format!("Mistral OCR response parsing failed: {e}")))
    }

    /// Concatène le markdown de toutes les pages, filigrane retiré.
    pub fn extract_markdown(response: &OcrResponse) -> String {
        if response.pages.is_empty() {
            return String::new();
        }
        let mut out = String::new();
        for page in &response.pages {
            out.push_str(&page.markdown);
            out.push('\n');
        }
        out.replace(WATERMARK, "")
    }

    /// Variante « best-effort » pour les pipelines RAG : renvoie une chaîne vide en cas d'échec
    /// (clé absente, réseau, PDF illisible) au lieu de propager l'erreur.
    pub async fn ocr_to_markdown(bytes: &[u8], filename: Option<&str>, mime: Option<&str>) -> String {
        match Self::perform_ocr(bytes, filename, mime).await {
            Ok(response) => Self::extract_markdown(&response),
            Err(e) => {
                tracing::warn!(file = filename.unwrap_or(DEFAULT_FILENAME), error = %e, "Mistral OCR fallback unavailable");
                String::new()
            }
        }
    }
}
