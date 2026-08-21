//! Endpoint d'OCR de documents, équivalent du `/api/mistral-ocr/process` de l'application Spring.
//!
//! Le fichier est reçu en `multipart/form-data` (champ `file`, comme le front Angular existant),
//! passé à l'API OCR Mistral, et le markdown extrait est renvoyé immédiatement.

use axum::{extract::Multipart, routing::post, Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use crate::errors::AppError;
use crate::services::mistral_ocr_service::MistralOcrService;
use crate::AppState;

/// Taille maximale acceptée, alignée sur `spring.servlet.multipart.max-file-size` (50 Mo).
const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new().route("/process", post(process))
}

/// Réponse de l'OCR : identifiant de document, nom de fichier et markdown extrait.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OcrProcessResponse {
    pub doc_id: String,
    pub filename: String,
    pub raw_text: String,
}

/// OCR d'un document (PDF ou image) et extraction du texte en markdown.
#[utoipa::path(
    post,
    path = "/api/mistral-ocr/process",
    tag = "mistral-ocr",
    request_body(content = String, description = "multipart/form-data avec un champ `file`", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Texte extrait du document", body = OcrProcessResponse),
        (status = 400, description = "Fichier manquant ou trop volumineux"),
        (status = 500, description = "Erreur de l'API OCR Mistral")
    )
)]
pub async fn process(mut multipart: Multipart) -> Result<Json<OcrProcessResponse>, AppError> {
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart payload: {e}")))?
    {
        // On ne retient que le champ `file` ; les autres champs éventuels sont ignorés.
        if field.name() != Some("file") {
            continue;
        }
        filename = field.file_name().map(str::to_owned);
        content_type = field.content_type().map(str::to_owned);
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Could not read uploaded file: {e}")))?;
        if data.len() > MAX_UPLOAD_BYTES {
            return Err(AppError::BadRequest(format!(
                "File too large: {} bytes (max {MAX_UPLOAD_BYTES})",
                data.len()
            )));
        }
        bytes = Some(data.to_vec());
        break;
    }

    let bytes = bytes.ok_or_else(|| AppError::BadRequest("Missing 'file' part in the request".into()))?;
    if bytes.is_empty() {
        return Err(AppError::BadRequest("Uploaded file is empty".into()));
    }

    let result = MistralOcrService::process(&bytes, filename.as_deref(), content_type.as_deref()).await?;

    Ok(Json(OcrProcessResponse {
        doc_id: result.doc_id,
        filename: result.filename,
        raw_text: result.raw_text,
    }))
}
