//! Async data loading for the TUI.
//!
//! Fetches are spawned on a tokio runtime and results are sent back via a channel
//! so the UI loop stays responsive.

use anyhow::Result;
use comma_common::{FileKind, missing_route_paths};
use comma_connect_api::{Client, Device, Route, RouteFiles, UploadFileRequest, UploadQueueItem};
use std::sync::mpsc::{self, Receiver, Sender};
use tokio::runtime::Handle;

pub enum DataMsg {
    Devices(Result<Vec<Device>>),
    Routes(Result<Vec<Route>>),
    RouteFiles(String, Result<RouteFiles>),
    Queue(Result<Vec<UploadQueueItem>>),
    DeviceDetail(Result<Device>),
}

pub struct Loader {
    client: Client,
    tx: Sender<DataMsg>,
    handle: Handle,
}

impl Loader {
    pub fn new(client: Client, tx: Sender<DataMsg>, handle: Handle) -> Self {
        Self { client, tx, handle }
    }

    pub fn load_devices(&self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        self.handle.spawn(async move {
            let res = client.get_devices().await.map_err(|e| anyhow::anyhow!(e));
            let _ = tx.send(DataMsg::Devices(res));
        });
    }

    pub fn load_routes(&self, dongle_id: String, limit: u32, fully_uploaded: std::collections::HashSet<String>) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        self.handle.spawn(async move {
            let res = client
                .get_routes(&dongle_id, Some(limit), None)
                .await
                .map_err(|e| anyhow::anyhow!(e));
            if let Ok(routes) = &res {
                for r in routes {
                    if fully_uploaded.contains(&r.fullname) {
                        continue; // already known fully uploaded; skip API query
                    }
                    let c = client.clone();
                    let t = tx.clone();
                    let name = r.fullname.clone();
                    tokio::spawn(async move {
                        let files = c.get_route_files(&name).await.map_err(|e| anyhow::anyhow!(e));
                        let _ = t.send(DataMsg::RouteFiles(name, files));
                    });
                }
            }
            let _ = tx.send(DataMsg::Routes(res));
        });
    }

    /// Force a fresh files query for one route (used by Ctrl-r refresh).
    pub fn load_route_files(&self, route_name: String) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        self.handle.spawn(async move {
            let files = client.get_route_files(&route_name).await.map_err(|e| anyhow::anyhow!(e));
            let _ = tx.send(DataMsg::RouteFiles(route_name, files));
        });
    }

    pub fn load_queue(&self, dongle_id: String) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        self.handle.spawn(async move {
            let res = client
                .list_upload_queue(&dongle_id)
                .await
                .map_err(|e| anyhow::anyhow!(e));
            let _ = tx.send(DataMsg::Queue(res));
        });
    }

    pub fn load_device_detail(&self, dongle_id: String) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        self.handle.spawn(async move {
            let res = client
                .get_device(&dongle_id)
                .await
                .map_err(|e| anyhow::anyhow!(e));
            let _ = tx.send(DataMsg::DeviceDetail(res));
        });
    }

    /// Load all data for a given dongle (routes + queue + detail).
    pub fn load_for_dongle(&self, dongle_id: &str, fully_uploaded: std::collections::HashSet<String>) {
        self.load_routes(dongle_id.to_string(), 20, fully_uploaded);
        self.load_queue(dongle_id.to_string());
        self.load_device_detail(dongle_id.to_string());
    }

    /// Request upload of one file kind for a single route (missing segments only).
    pub fn request_upload_kind(&self, dongle_id: String, route_name: String, kind: FileKind) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        self.handle.spawn(async move {
            let res = do_upload_kind(&client, &dongle_id, &route_name, kind).await;
            // Reuse Queue message channel? For now just log via a dummy; we will
            // send a Routes refresh or a dedicated message. Simplest: log and
            // trigger a queue reload by sending Queue(Err) is wrong.
            // We'll send nothing special; the caller reloads the queue.
            let _ = res;
            let _ = tx; // keep tx available if we later add a status message
        });
    }

    /// Request upload of all missing file kinds for one route.
    pub fn request_upload_route(&self, dongle_id: String, route_name: String) {
        let client = self.client.clone();
        self.handle.spawn(async move {
            let _ = do_upload_route(&client, &dongle_id, &route_name).await;
        });
    }

    /// Request upload of all missing files for every given route.
    pub fn request_upload_all(&self, dongle_id: String, routes: Vec<String>) {
        let client = self.client.clone();
        self.handle.spawn(async move {
            for name in routes {
                let _ = do_upload_route(&client, &dongle_id, &name).await;
            }
        });
    }

    /// Cancel one queue item by id.
    pub fn cancel_selected(&self, dongle_id: String, item_id: String) {
        let client = self.client.clone();
        self.handle.spawn(async move {
            let _ = client.cancel_upload(&dongle_id, std::slice::from_ref(&item_id)).await;
        });
    }

    /// Cancel all pending (non-current) uploads.
    pub fn cancel_pending(&self, dongle_id: String) {
        let client = self.client.clone();
        self.handle.spawn(async move {
            let _ = client.cancel_pending_uploads(&dongle_id).await;
        });
    }
}

async fn do_upload_kind(client: &Client, dongle_id: &str, route_name: &str, kind: FileKind) -> anyhow::Result<()> {
    let route = client.get_route(route_name).await.map_err(|e| anyhow::anyhow!(e))?;
    let segment_count = route.maxqlog.map(|m| m + 1).unwrap_or(0);
    if segment_count <= 0 {
        return Ok(());
    }
    let files = client.get_route_files(route_name).await.map_err(|e| anyhow::anyhow!(e))?;
    let already = kind.uploaded(&files);
    let route_ts = route_name.split('|').nth(1).unwrap_or(route_name);
    let mut paths = Vec::new();
    for n in 0..segment_count {
        if (n as usize) < already.len() {
            continue;
        }
        paths.push(format!("{route_ts}--{n}/{}", kind.filename()));
    }
    if paths.is_empty() {
        return Ok(());
    }
    let urls = client.get_upload_urls(dongle_id, &paths, Some(7)).await.map_err(|e| anyhow::anyhow!(e))?;
    if urls.len() != paths.len() {
        return Err(anyhow::anyhow!("url count mismatch"));
    }
    let requests: Vec<UploadFileRequest> = paths.into_iter().zip(urls).map(|(fn_, u)| UploadFileRequest {
        file_name: fn_,
        url: u.url,
        headers: u.headers.unwrap_or_default(),
        allow_cellular: false,
        priority: 1,
    }).collect();
    client.upload_files_to_urls(dongle_id, &requests, None).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

async fn do_upload_route(client: &Client, dongle_id: &str, route_name: &str) -> anyhow::Result<()> {
    let route = client.get_route(route_name).await.map_err(|e| anyhow::anyhow!(e))?;
    let segment_count = route.maxqlog.map(|m| m + 1).unwrap_or(0);
    if segment_count <= 0 {
        return Ok(());
    }
    let files = client.get_route_files(route_name).await.map_err(|e| anyhow::anyhow!(e))?;
    let route_ts = route_name.split('|').nth(1).unwrap_or(route_name);
    let paths = missing_route_paths(route_ts, segment_count, &files, None);
    if paths.is_empty() {
        return Ok(());
    }
    let urls = client.get_upload_urls(dongle_id, &paths, Some(7)).await.map_err(|e| anyhow::anyhow!(e))?;
    if urls.len() != paths.len() {
        return Err(anyhow::anyhow!("url count mismatch"));
    }
    let requests: Vec<UploadFileRequest> = paths.into_iter().zip(urls).map(|(fn_, u)| UploadFileRequest {
        file_name: fn_,
        url: u.url,
        headers: u.headers.unwrap_or_default(),
        allow_cellular: false,
        priority: 1,
    }).collect();
    client.upload_files_to_urls(dongle_id, &requests, None).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

pub fn channel() -> (Sender<DataMsg>, Receiver<DataMsg>) {
    mpsc::channel()
}
