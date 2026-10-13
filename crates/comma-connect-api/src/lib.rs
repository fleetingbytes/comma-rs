mod error;

pub use error::{Error, Result};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, warn};

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
        debug!(url = %url, "sending GET request");
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
            warn!(status = %status, body = %body, "API request failed");
            return Err(Error::Api {
                status: status.as_u16(),
                body,
            });
        }

        info!(status = %status, path = %path, "API request succeeded");
        debug!(body_len = body.len(), "response body received");

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

    /// List routes that have been uploaded from a device (minimally or fully).
    ///
    /// `created_after` is a Unix timestamp in milliseconds (optional).
    /// Returns routes ordered by creation time (newest first, per API).
    pub async fn get_routes(
        &self,
        dongle_id: &str,
        limit: Option<u32>,
        created_after: Option<i64>,
    ) -> Result<Vec<Route>> {
        let mut path = format!("/v1/devices/{}/routes", dongle_id);
        let mut query = Vec::new();
        if let Some(l) = limit {
            query.push(format!("limit={}", l));
        }
        if let Some(ts) = created_after {
            query.push(format!("created_after={}", ts));
        }
        if !query.is_empty() {
            path.push('?');
            path.push_str(&query.join("&"));
        }
        let value = self.get(&path).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// List the segments that make up a route.
    ///
    /// Each segment includes file-processing status (`proc_log`, `proc_camera`, etc.).
    /// Route name must be the full form `dongle_id|YYYY-MM-DD--HH-MM-SS` (or equivalent).
    /// The `|` is percent-encoded automatically.
    pub async fn get_route_segments(&self, route_name: &str) -> Result<Vec<Segment>> {
        let encoded = route_name.replace('|', "%7C");
        let path = format!("/v1/route/{}/segments", encoded);
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
    #[serde(flatten)]
    pub extra: Value,
}

/// A route that has been (at least partially) uploaded to the comma servers.
/// `create_time` is when the first upload URL was requested for the route.
/// Note: some timestamp fields arrive as ISO-8601 strings rather than numeric Unix timestamps.
#[derive(Debug, Serialize, Deserialize)]
pub struct Route {
    pub fullname: String,
    pub dongle_id: String,
    #[serde(default)]
    pub create_time: Option<i64>,
    #[serde(default)]
    pub length: Option<f64>,
    #[serde(default)]
    pub is_public: Option<bool>,
    #[serde(default)]
    pub maxlog: Option<i32>,
    #[serde(default)]
    pub maxcamera: Option<i32>,
    #[serde(default)]
    pub maxqlog: Option<i32>,
    #[serde(default)]
    pub maxqcamera: Option<i32>,
    #[serde(default)]
    pub proclog: Option<i32>,
    #[serde(default)]
    pub proccamera: Option<i32>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    /// Often an ISO-8601 string (e.g. "2026-10-11T15:24:41") rather than a Unix timestamp.
    #[serde(default)]
    pub start_time: Option<String>,
    /// Often an ISO-8601 string rather than a Unix timestamp.
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub start_lat: Option<f64>,
    #[serde(default)]
    pub start_lng: Option<f64>,
    #[serde(flatten)]
    pub extra: Value,
}

/// A single segment (up to ~60 s) of a route.
///
/// File status codes (`proc_*`):
/// - -1 = not received
/// -  0 = upload URL sent
/// - 10 = received
/// - 20 = enqueued
/// - 30 = processing
/// - 40 = processed
/// - 50 = errored
#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    pub canonical_name: String,
    pub number: i32,
    pub canonical_route_name: String,
    pub dongle_id: String,

    #[serde(default)]
    pub create_time: Option<i64>,
    #[serde(default)]
    pub start_time_utc_millis: Option<i64>,
    #[serde(default)]
    pub end_time_utc_millis: Option<i64>,

    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub length: Option<f64>,

    #[serde(default)]
    pub can: Option<bool>,
    #[serde(default)]
    pub hpgps: Option<bool>,
    #[serde(default)]
    pub radar: Option<bool>,
    #[serde(default)]
    pub passive: Option<bool>,

    #[serde(default)]
    pub devicetype: Option<i32>,

    /// Full log (`rlog`) processing status.
    #[serde(default)]
    pub proc_log: Option<i32>,
    /// Decimated log (`qlog`) processing status (nullable).
    #[serde(default)]
    pub proc_qlog: Option<i32>,
    /// Road camera processing status.
    #[serde(default)]
    pub proc_camera: Option<i32>,
    /// Driver camera processing status.
    #[serde(default)]
    pub proc_dcamera: Option<i32>,

    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub git_commit: Option<String>,
    #[serde(default)]
    pub git_branch: Option<String>,
    #[serde(default)]
    pub git_remote: Option<String>,
    #[serde(default)]
    pub git_dirty: Option<bool>,

    #[serde(default)]
    pub start_lat: Option<f64>,
    #[serde(default)]
    pub start_lng: Option<f64>,
    #[serde(default)]
    pub end_lat: Option<f64>,
    #[serde(default)]
    pub end_lng: Option<f64>,

    #[serde(flatten)]
    pub extra: Value,
}
