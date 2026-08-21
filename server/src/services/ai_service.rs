use rig::client::{AgentClientExt, ProviderClient};
use rig::completion::Prompt;
use rig::providers::mistral;

use crate::errors::AppError;

pub struct AiService;

impl AiService {
    /// Send a prompt to the Mistral agent using the MISTRAL_API_KEY environment variable
    pub async fn prompt(prompt_text: &str) -> Result<String, AppError> {
        Self::prompt_with_model(
            prompt_text,
            "mistral-small-latest",
            "You are a helpful assistant.",
        )
            .await
    }

    /// Send a prompt to the Mistral agent with a custom model and preamble
    pub async fn prompt_with_model(
        prompt_text: &str,
        model: &str,
        preamble: &str,
    ) -> Result<String, AppError> {
        // Create the Mistral client from the MISTRAL_API_KEY environment variable.
        let client = mistral::Client::from_env()
            .map_err(|e| {
                AppError::Internal(format!(
                    "Failed to initialize Mistral client: {}",
                    e
                ))
            })?;

        // Build an agent: a model plus a system prompt.
        let agent = client
            .agent(model)
            .preamble(preamble)
            .build();

        // Send a prompt and await the model's reply.
        let response = agent
            .prompt(prompt_text)
            .await
            .map_err(|e| {
                AppError::Internal(format!(
                    "Mistral AI prompt error: {}",
                    e
                ))
            })?;

        Ok(response)
    }
}