//! Assistant conversationnel outille (`/api/job-copilot/assistant`).
//!
//! Deux surfaces sur le meme agent : un tour bloquant qui renvoie du JSON, et un tour diffuse en
//! Server-Sent Events.
//!
//! Le format des trames n'est pas libre : le front ne passe pas par `EventSource`. Il fait un POST
//! `responseType: 'text'` avec `reportProgress`, accumule `partialText` et le passe a son propre
//! `parseSSEWithMeta`. Ce parseur decoupe sur `\n\n`, ne lit que les lignes `data:`, saute les
//! types `start` et `end`, et concatene le champ `content` de tout le reste. Emettre du texte brut
//! afficherait donc la reponse sans probleme apparent mais casserait la gestion des sources ;
//! emettre un autre nom de champ que `content` afficherait une bulle vide.

use std::convert::Infallible;

use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::post,
    Extension, Json, Router,
};
use futures::stream::{Stream, StreamExt};
use rig::agent::MultiTurnStreamItem;
use rig::completion::Prompt;
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
use serde::Serialize;
use utoipa::ToSchema;

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::RoleType;
use crate::services::job_agent_service::JobAgentService;
use crate::AppState;

/// Intervalle des commentaires de maintien de connexion, aligne sur la version Java.
///
/// Un proxy ferme volontiers une connexion inactive, et le modele peut rester silencieux plusieurs
/// secondes pendant un appel d'outil. Les trames de maintien sont des commentaires SSE : elles ne
/// commencent pas par `data:`, donc le parseur du front les ignore.
const KEEP_ALIVE_SECS: u64 = 15;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:id/message", post(message))
        .route("/:id/stream", post(stream))
}

/// Un tour d'assistant, forme attendue par `AssistantMessage` cote Angular.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessageDto {
    pub conversation_id: String,
    pub message: String,
}

/// Tour bloquant : la reponse complete en une fois.
#[utoipa::path(
    post,
    path = "/api/job-copilot/assistant/{id}/message",
    tag = "job-copilot-assistant",
    security(("bearer_auth" = [])),
    params(("id" = String, Path, description = "Identifiant de conversation")),
    request_body(content = String, description = "Message de l'utilisateur", content_type = "text/plain"),
    responses(
        (status = 200, description = "Reponse de l'assistant", body = AssistantMessageDto),
        (status = 400, description = "Message vide"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn message(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    body: String,
) -> Result<Json<AssistantMessageDto>, AppError> {
    let (conversation_id, prompt) = validate(&auth, conversation_id, body)?;

    let agent = JobAgentService::build_agent(
        state.pool.clone(),
        state.job_offer_index.clone(),
        &auth.login,
    )?;

    // `conversation` porte la cle de memoire : sans elle, rig contourne silencieusement la
    // memoire et l'assistant repartirait de zero a chaque message.
    let reply = agent
        .prompt(prompt)
        .conversation(&conversation_id)
        .await
        .map_err(|e| AppError::Internal(format!("Echec de l'agent : {e}")))?;

    Ok(Json(AssistantMessageDto {
        conversation_id,
        message: reply,
    }))
}

/// Tour diffuse en SSE.
#[utoipa::path(
    post,
    path = "/api/job-copilot/assistant/{id}/stream",
    tag = "job-copilot-assistant",
    security(("bearer_auth" = [])),
    params(("id" = String, Path, description = "Identifiant de conversation")),
    request_body(content = String, description = "Message de l'utilisateur", content_type = "text/plain"),
    responses(
        (status = 200, description = "Flux d'evenements start / token* / end", content_type = "text/event-stream"),
        (status = 400, description = "Message vide"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn stream(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(conversation_id): Path<String>,
    body: String,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let (conversation_id, prompt) = validate(&auth, conversation_id, body)?;

    let agent = JobAgentService::build_agent(
        state.pool.clone(),
        state.job_offer_index.clone(),
        &auth.login,
    )?;

    // Le flux est ouvert ici, avant de rendre la reponse : une panne de construction doit
    // ressortir en erreur HTTP franche, pas en flux vide que le front afficherait comme une
    // reponse vide de l'assistant.
    let inner = agent
        .stream_prompt(prompt)
        .conversation(&conversation_id)
        .await;

    let events = futures::stream::once(async { Ok(chat_event(START, None)) })
        .chain(inner.map(|item| {
            Ok(match item {
                // Seuls les deltas de texte alimentent la bulle. Les appels d'outils et leurs
                // resultats traversent le flux mais n'ont rien a afficher : les emettre en
                // `content` ferait apparaitre du JSON d'outil dans la conversation.
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(
                    text,
                ))) => chat_event(TOKEN, Some(&text.text)),
                Ok(_) => keep_alive_event(),
                // L'erreur est diffusee dans le flux plutot que de couper la connexion : le
                // parseur du front concatene le `content` des types inconnus, donc l'utilisateur
                // lit le message au lieu de voir la bulle rester vide.
                Err(e) => {
                    tracing::warn!(error = %e, "Flux de l'assistant interrompu");
                    chat_event(ERROR, Some("La reponse a ete interrompue."))
                }
            })
        }))
        .chain(futures::stream::once(async { Ok(chat_event(END, None)) }));

    Ok(Sse::new(events).keep_alive(
        KeepAlive::new().interval(std::time::Duration::from_secs(KEEP_ALIVE_SECS)),
    ))
}

/// Charge utile d'une trame de chat.
///
/// Fonction distincte de [`chat_event`] pour qu'elle soit testable : `Event` n'expose pas son
/// contenu, donc tester la trame elle-meme obligerait a la relire par sa representation `Debug`.
/// C'est cette charge utile que le parseur du front lit, et `type` / `content` sont les deux seuls
/// noms qu'il reconnait.
fn chat_payload(kind: &str, content: Option<&str>) -> serde_json::Value {
    match content {
        Some(content) => serde_json::json!({ "type": kind, "content": content }),
        None => serde_json::json!({ "type": kind }),
    }
}

/// Une trame `data:` porteuse d'un evenement de chat.
fn chat_event(kind: &str, content: Option<&str>) -> Event {
    Event::default().data(chat_payload(kind, content).to_string())
}

/// Trame sans contenu, pour les items du flux qui ne portent pas de texte.
///
/// Emise comme un `ping` : le parseur du front lit son `content` absent comme une chaine vide, ce
/// qui ne modifie pas le texte affiche, tout en gardant la connexion vivante pendant un appel
/// d'outil long.
fn keep_alive_event() -> Event {
    chat_event(PING, None)
}

/// Types de trames. Nommes pour que le prompt du front et le serveur ne divergent pas en silence.
const START: &str = "start";
const TOKEN: &str = "token";
const PING: &str = "ping";
const END: &str = "end";
const ERROR: &str = "error";

/// Verifie le role, l'identifiant de conversation et le message.
fn validate(
    auth: &AuthUser,
    conversation_id: String,
    body: String,
) -> Result<(String, String), AppError> {
    if !auth.has_authority(RoleType::USER) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let conversation_id = conversation_id.trim().to_string();
    if conversation_id.is_empty() {
        // Le front construit l'URL avec `conversationId ?? ''` : un identifiant absent donnerait
        // un segment vide, et rig contournerait la memoire sans rien signaler.
        return Err(AppError::BadRequest(
            "Identifiant de conversation manquant".into(),
        ));
    }

    let prompt = body.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::BadRequest("Message vide".into()));
    }

    Ok((conversation_id, prompt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_payload_uses_the_field_names_the_front_parses() {
        // Le parseur du front lit `type` et `content`, rien d'autre : tout autre nom afficherait
        // une bulle vide.
        let payload = chat_payload(TOKEN, Some("Bonjour"));
        assert_eq!(payload["type"], "token");
        assert_eq!(payload["content"], "Bonjour");
    }

    #[test]
    fn start_and_end_payloads_carry_no_content() {
        // Le front saute explicitement `start` et `end` ; leur donner un `content` le ferait
        // apparaitre dans la conversation.
        for kind in [START, END] {
            let payload = chat_payload(kind, None);
            assert_eq!(payload["type"], kind);
            assert!(
                payload.get("content").is_none(),
                "{kind} ne doit porter aucun contenu"
            );
        }
    }

    #[test]
    fn ping_payload_adds_no_text() {
        let payload = chat_payload(PING, None);
        assert_eq!(payload["type"], "ping");
        assert!(payload.get("content").is_none());
    }

    #[test]
    fn error_payload_is_readable_by_the_user() {
        // Le parseur du front concatene le `content` des types qu'il ne connait pas : un message
        // d'erreur y est donc lu, au lieu de laisser la bulle vide.
        let payload = chat_payload(ERROR, Some("La reponse a ete interrompue."));
        assert_eq!(payload["type"], "error");
        assert_eq!(payload["content"], "La reponse a ete interrompue.");
    }

    #[test]
    fn a_rendered_frame_survives_the_front_parser() {
        // Reproduction du decoupage fait par `parseSSEWithMeta` : separation sur `\n\n`, lignes
        // `data:` seules, concatenation des `content` hors `start` et `end`.
        let wire = format!(
            "data: {}\n\ndata: {}\n\ndata: {}\n\ndata: {}\n\n",
            chat_payload(START, None),
            chat_payload(TOKEN, Some("Bon")),
            chat_payload(TOKEN, Some("jour")),
            chat_payload(END, None),
        );

        let mut text = String::new();
        for frame in wire.split("\n\n") {
            for line in frame.lines() {
                let line = line.trim_start();
                let Some(payload) = line.strip_prefix("data:") else {
                    continue;
                };
                let payload = payload.trim();
                if payload.is_empty() {
                    continue;
                }
                let json: serde_json::Value = serde_json::from_str(payload).unwrap();
                let kind = json["type"].as_str().unwrap_or_default();
                if kind == "start" || kind == "end" {
                    continue;
                }
                text.push_str(json["content"].as_str().unwrap_or_default());
            }
        }

        assert_eq!(text, "Bonjour", "les jetons doivent se recoller sans bruit");
    }

    #[test]
    fn validate_rejects_an_empty_conversation_id() {
        // Le front envoie `conversationId ?? ''` : sans ce garde-fou, rig contournerait la
        // memoire en silence et l'assistant perdrait le fil sans erreur visible.
        let auth = AuthUser {
            login: "alice".to_string(),
            authorities: vec![RoleType::USER.to_string()],
        };
        let error = validate(&auth, "   ".to_string(), "bonjour".to_string()).unwrap_err();
        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn validate_rejects_an_empty_message() {
        let auth = AuthUser {
            login: "alice".to_string(),
            authorities: vec![RoleType::USER.to_string()],
        };
        let error = validate(&auth, "conv-1".to_string(), "  \n ".to_string()).unwrap_err();
        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn validate_requires_the_user_role() {
        let anonymous = AuthUser::anonymous();
        let error = validate(&anonymous, "conv-1".to_string(), "bonjour".to_string()).unwrap_err();
        assert!(matches!(error, AppError::Forbidden(_)));
    }

    #[test]
    fn validate_trims_both_inputs() {
        let auth = AuthUser {
            login: "alice".to_string(),
            authorities: vec![RoleType::USER.to_string()],
        };
        let (conversation, prompt) =
            validate(&auth, "  conv-1  ".to_string(), "  bonjour  ".to_string()).unwrap();
        assert_eq!(conversation, "conv-1");
        assert_eq!(prompt, "bonjour");
    }
}
