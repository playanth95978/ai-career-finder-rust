//! Accès au modèle de chat Mistral via RIG.
//!
//! Le modèle par défaut est aligné sur celui de l'application Spring d'origine
//! (`mistral-medium-3.5`) et surchargeable par la variable d'environnement
//! `MISTRAL_CHAT_MODEL`, comme `spring.ai.mistralai.chat.model` côté Java.

use rig::client::{AgentClientExt, ProviderClient};
use rig::completion::Prompt;
use rig::providers::mistral;

use crate::errors::AppError;

/// Modèle de chat par défaut — identique à celui de l'app Spring.
const DEFAULT_CHAT_MODEL: &str = "mistral-medium-3.5";
/// Modèle de repli, utilisé côté Java quand le principal renvoie 429.
pub const FALLBACK_CHAT_MODEL: &str = "mistral-small-latest";
/// Préambule (system prompt) par défaut.
pub const DEFAULT_PREAMBLE: &str = "You are a helpful assistant.";

pub struct AiService;

impl AiService {
    /// Modèle de chat effectif : `MISTRAL_CHAT_MODEL` sinon [`DEFAULT_CHAT_MODEL`].
    pub fn default_model() -> String {
        std::env::var("MISTRAL_CHAT_MODEL")
            .ok()
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_CHAT_MODEL.to_string())
    }

    /// Envoie un prompt au modèle par défaut (clé lue dans `MISTRAL_API_KEY`).
    pub async fn prompt(prompt_text: &str) -> Result<String, AppError> {
        Self::prompt_with_model(prompt_text, &Self::default_model(), DEFAULT_PREAMBLE).await
    }

    /// Envoie un prompt en choisissant explicitement le modèle et le préambule.
    pub async fn prompt_with_model(
        prompt_text: &str,
        model: &str,
        preamble: &str,
    ) -> Result<String, AppError> {
        // Client Mistral construit depuis MISTRAL_API_KEY.
        let client = mistral::Client::from_env()
            .map_err(|e| AppError::Internal(format!("Failed to initialize Mistral client: {e}")))?;

        // Un agent = un modèle + un system prompt.
        let agent = client.agent(model).preamble(preamble).build();

        agent
            .prompt(prompt_text)
            .await
            .map_err(|e| AppError::Internal(format!("Mistral AI prompt error: {e}")))
    }
}
