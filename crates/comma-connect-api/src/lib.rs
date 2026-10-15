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
    /// `created_after` is a Unix timestamp in seconds (optional).
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

    /// Retrieve the uploaded files for a route.
    ///
    /// Returns signed URLs (valid ~1 hour) for each file type, indexed by segment number.
    /// An empty array for a type means no segments of that type have been uploaded.
    /// Note: the previously documented `/v1/route/{name}/segments` endpoint returns 404
    /// on the live API; this is the practical source of per-segment upload status.
    /// The `|` in the route name is percent-encoded automatically.
    pub async fn get_route_files(&self, route_name: &str) -> Result<RouteFiles> {
        let encoded = route_name.replace('|', "%7C");
        let path = format!("/v1/route/{}/files", encoded);
        let value = self.get(&path).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Fetch summary information for a single route (includes `maxqlog`, etc.).
    pub async fn get_route(&self, route_name: &str) -> Result<Route> {
        let encoded = route_name.replace('|', "%7C");
        let path = format!("/v1/route/{}/", encoded);
        let value = self.get(&path).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Query the device's current upload queue via Athena (`listUploadQueue`).
    ///
    /// Returns the files currently queued or actively uploading, including a
    /// `progress` value in the range 0.0–1.0. Requires the device to be online;
    /// if it is offline the Athena response will indicate that (or the call may fail).
    pub async fn list_upload_queue(&self, dongle_id: &str) -> Result<Vec<UploadQueueItem>> {
        const ATHENA_URL: &str = "https://athena.comma.ai";
        let url = format!("{}/{}", ATHENA_URL, dongle_id);
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "listUploadQueue",
            "params": {},
            "id": 0
        });
        debug!(url = %url, "sending Athena listUploadQueue");
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("JWT {}", self.token))
            .header("User-Agent", "comma-rs/0.1.0")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            warn!(status = %status, body = %text, "Athena request failed");
            return Err(Error::Api {
                status: status.as_u16(),
                body: text,
            });
        }

        let value: Value = serde_json::from_str(&text)?;
        // Athena returns { "jsonrpc": "...", "id": ..., "result": [...] }
        // or an error object.
        if let Some(err) = value.get("error") {
            return Err(Error::Api {
                status: status.as_u16(),
                body: err.to_string(),
            });
        }
        let result = value.get("result").cloned().unwrap_or(Value::Array(vec![]));
        Ok(serde_json::from_value(result)?)
    }

    /// Cancel one or more items in the device's Athena upload queue.
    ///
    /// `ids` are the `id` values from `list_upload_queue`. The device only
    /// removes items that are still waiting in the queue (not the one currently
    /// uploading). Returns the Athena JSON-RPC result object.
    pub async fn cancel_upload(&self, dongle_id: &str, ids: &[String]) -> Result<Value> {
        const ATHENA_URL: &str = "https://athena.comma.ai";
        let url = format!("{}/{}", ATHENA_URL, dongle_id);
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "cancelUpload",
            "params": {
                "upload_id": ids
            },
            "id": 0
        });
        debug!(url = %url, count = ids.len(), "sending Athena cancelUpload");
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("JWT {}", self.token))
            .header("User-Agent", "comma-rs/0.1.0")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            warn!(status = %status, body = %text, "Athena cancelUpload failed");
            return Err(Error::Api {
                status: status.as_u16(),
                body: text,
            });
        }

        let value: Value = serde_json::from_str(&text)?;
        if let Some(err) = value.get("error") {
            return Err(Error::Api {
                status: status.as_u16(),
                body: err.to_string(),
            });
        }
        Ok(value)
    }

    /// Cancel all pending (non-current) items in the upload queue.
    ///
    /// Lists the queue, collects ids of items where `current` is not true,
    /// and calls `cancel_upload` with those ids. Returns the number of ids
    /// that were submitted for cancellation (0 if none).
    pub async fn cancel_pending_uploads(&self, dongle_id: &str) -> Result<usize> {
        let queue = self.list_upload_queue(dongle_id).await?;
        let ids: Vec<String> = queue
            .into_iter()
            .filter(|item| item.current != Some(true))
            .filter_map(|item| item.id)
            .collect();
        if ids.is_empty() {
            return Ok(0);
        }
        self.cancel_upload(dongle_id, &ids).await?;
        Ok(ids.len())
    }

    /// Request signed upload URLs for a list of device-relative paths.
    ///
    /// `paths` are of the form `{route_ts}--{segment}/filename`, e.g.
    /// `00000148--846422f721--0/fcamera.hevc`.
    /// `expiry_days` must be between 1 and 30 (default 7).
    pub async fn get_upload_urls(
        &self,
        dongle_id: &str,
        paths: &[String],
        expiry_days: Option<u32>,
    ) -> Result<Vec<UploadUrl>> {
        let path = format!("/v1/{}/upload_urls/", dongle_id);
        let body = serde_json::json!({
            "paths": paths,
            "expiry_days": expiry_days.unwrap_or(7)
        });
        debug!(dongle_id, count = paths.len(), "requesting upload URLs");
        let url = format!("{}{}", BASE_URL, path);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("JWT {}", self.token))
            .header("User-Agent", "comma-rs/0.1.0")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            warn!(status = %status, body = %text, "upload_urls request failed");
            return Err(Error::Api {
                status: status.as_u16(),
                body: text,
            });
        }
        Ok(serde_json::from_str(&text)?)
    }

    /// Ask the device (via Athena) to upload the given files to the provided URLs.
    ///
    /// `priority` 1 is the high priority used by comma connect.
    /// `expiry_seconds` is how long the request stays in the offline queue if the
    /// device is offline (default 7 days).
    pub async fn upload_files_to_urls(
        &self,
        dongle_id: &str,
        files: &[UploadFileRequest],
        expiry_seconds: Option<u64>,
    ) -> Result<Value> {
        const ATHENA_URL: &str = "https://athena.comma.ai";
        let url = format!("{}/{}", ATHENA_URL, dongle_id);
        let expiry = expiry_seconds.unwrap_or(60 * 60 * 24 * 7);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "uploadFilesToUrls",
            "params": {
                "files_data": serde_json::to_value(files)?
            },
            "id": 0,
            "expiry": now + expiry
        });
        debug!(dongle_id, count = files.len(), "sending Athena uploadFilesToUrls");
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("JWT {}", self.token))
            .header("User-Agent", "comma-rs/0.1.0")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            warn!(status = %status, body = %text, "Athena uploadFilesToUrls failed");
            return Err(Error::Api {
                status: status.as_u16(),
                body: text,
            });
        }
        let value: Value = serde_json::from_str(&text)?;
        if let Some(err) = value.get("error") {
            return Err(Error::Api {
                status: status.as_u16(),
                body: err.to_string(),
            });
        }
        Ok(value)
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

/// An item in the device's Athena upload queue.
///
/// `progress` is a fraction in [0.0, 1.0] (bytes sent / total size).
/// `current` is true while the item is actively being uploaded.
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadQueueItem {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub retry_count: Option<i32>,
    #[serde(default)]
    pub current: Option<bool>,
    /// Upload progress as a fraction 0.0–1.0.
    #[serde(default)]
    pub progress: Option<f64>,
    #[serde(default)]
    pub allow_cellular: Option<bool>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Uploaded files for a route, grouped by type.
/// Each array is ordered by segment number; a missing or empty array means
/// that file type has not been uploaded for any segment.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RouteFiles {
    #[serde(default)]
    pub cameras: Vec<String>,
    #[serde(default)]
    pub dcameras: Vec<String>,
    #[serde(default)]
    pub ecameras: Vec<String>,
    #[serde(default)]
    pub logs: Vec<String>,
    #[serde(default)]
    pub qlogs: Vec<String>,
    #[serde(default)]
    pub qcameras: Vec<String>,
    #[serde(flatten)]
    pub extra: Value,
}

/// A signed URL the device can PUT a file to.
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadUrl {
    pub url: String,
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// One file the device should upload, as expected by Athena `uploadFilesToUrls`.
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadFileRequest {
    #[serde(rename = "fn")]
    pub file_name: String,
    pub url: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub allow_cellular: bool,
    /// Lower number = higher priority. comma connect uses 1.
    pub priority: i32,
}
