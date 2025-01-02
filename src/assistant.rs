use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs,
};
pub use async_openai::Client;
use dotenvy::dotenv;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chat::{Sender, TransportMessage};
use crate::error;

#[derive(Debug, Clone)]
pub struct Response {
    pub content: String,
    pub timestamp: u64,
}

pub fn client(
) -> Result<Client<async_openai::config::OpenAIConfig>, error::AppError> {
    dotenv().map_err(error::AppError::EnvLoadError)?;

    let api_key = env::var("OPENAI_KEY")?;
    let config =
        async_openai::config::OpenAIConfig::new().with_api_key(api_key);

    Ok(Client::with_config(config))
}

pub async fn request(
    client: Client<async_openai::config::OpenAIConfig>,
    history: &[TransportMessage],
    new_message: TransportMessage,
) -> Result<Response, String> {
    let mut openai_messages = vec![ChatCompletionRequestSystemMessage::from(
        "You are a helpful AI assistant. Be concise and clear in your responses.",
    )
    .into()];

    for msg in history {
        let openai_msg = match msg.sender {
            Sender::User => {
                ChatCompletionRequestUserMessage::from(msg.content.clone())
                    .into()
            }
            Sender::Assistant => {
                ChatCompletionRequestAssistantMessage::from(msg.content.clone())
                    .into()
            }
        };
        openai_messages.push(openai_msg);
    }

    openai_messages.push(
        ChatCompletionRequestUserMessage::from(new_message.content).into(),
    );

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(1024u32)
        .model("gpt-4")
        .messages(openai_messages)
        .build()
        .map_err(|e| format!("Failed to build request: {}", e))?;

    let response = client
        .chat()
        .create(request)
        .await
        .map_err(|e| format!("Failed to communicate with OpenAI: {}", e))?;

    let content = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .ok_or_else(|| "No content in OpenAI response".to_string())?;

    Ok(Response {
        content,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}
