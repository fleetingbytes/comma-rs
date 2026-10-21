use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },

    /// Device is not registered with Athena (typically powered off / offline).
    /// Not a real error; callers should treat the device as offline.
    #[error("device offline (not registered)")]
    DeviceOffline,

    #[error("Failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
