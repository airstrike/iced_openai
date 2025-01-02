use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Error loading .env file ({0}).\nPlease create a .env file in the project root directory with the following content:\nOPENAI_KEY=your-api-key\n")]
    EnvLoadError(#[from] dotenvy::Error),

    #[error("OpenAI API key not found in environment variables: {0}")]
    ApiKeyMissing(#[from] std::env::VarError),
}
