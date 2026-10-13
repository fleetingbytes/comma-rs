mod error;

pub use error::{Error, Result};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const BASE_URL: &str = "https://api.commadotai.com";

#[derive(Debug, Clone)]
pub struct Client {
    token: String,
    http: reqwest::Client,
}

impl Client {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", BASE_URL, path);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("JWT {}", self.token))
            .header("User-Agent", "comma-rs/0.1.0")
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(Error::Api {
                status: status.as_u16(),
                body,
            });
        }

        Ok(serde_json::from_str(&body)?)
    }

    /// Fetch the authenticated user's profile.
    pub async fn get_me(&self) -> Result<Profile> {
        let value = self.get("/v1/me/").await?;
        Ok(serde_json::from_value(value)?)
    }

    /// List devices accessible to the authenticated user.
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        let value = self.get("/v1/me/devices/").await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Get detailed information for a specific device.
    pub async fn get_device(&self, dongle_id: &str) -> Result<Device> {
        let path = format!("/v1.1/devices/{}/", dongle_id);
        let value = self.get(&path).await?;
        Ok(serde_json::from_value(value)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub email: Option<String>,
    pub id: String,
    #[serde(default)]
    pub points: Option<i64>,
    pub regdate: i64,
    pub superuser: bool,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    // Capture any additional/unknown fields so future API changes don't break parsing
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub dongle_id: String,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub serial: Option<String>,
    #[serde(default)]
    pub device_type: Option<String>,
    pub is_owner: bool,
    pub is_paired: bool,
    pub prime: bool,
    #[serde(default)]
    pub prime_type: Option<i32>,
    #[serde(default)]
    pub openpilot_version: Option<String>,
    #[serde(default)]
    pub last_gps_lat: Option<f64>,
    #[serde(default)]
    pub last_gps_lng: Option<f64>,
    #[serde(default)]
    pub last_gps_time: Option<i64>,
    // Allow extra fields without failing
    #[serde(flatten)]
    pub extra: Value,
}
