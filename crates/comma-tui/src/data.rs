//! Async data loading for the TUI.
//!
//! Fetches are spawned on a tokio runtime and results are sent back via a channel
//! so the UI loop stays responsive.

use anyhow::Result;
use comma_connect_api::{Client, Device, Route, UploadQueueItem};
use std::sync::mpsc::{self, Receiver, Sender};
use tokio::runtime::Handle;

pub enum DataMsg {
    Devices(Result<Vec<Device>>),
    Routes(Result<Vec<Route>>),
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

    pub fn load_routes(&self, dongle_id: String, limit: u32) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        self.handle.spawn(async move {
            let res = client
                .get_routes(&dongle_id, Some(limit), None)
                .await
                .map_err(|e| anyhow::anyhow!(e));
            let _ = tx.send(DataMsg::Routes(res));
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
    pub fn load_for_dongle(&self, dongle_id: &str) {
        self.load_routes(dongle_id.to_string(), 20);
        self.load_queue(dongle_id.to_string());
        self.load_device_detail(dongle_id.to_string());
    }
}

pub fn channel() -> (Sender<DataMsg>, Receiver<DataMsg>) {
    mpsc::channel()
}
