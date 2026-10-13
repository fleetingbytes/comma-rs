use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },

    #[error("Failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
