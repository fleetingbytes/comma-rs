mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use comma_connect_api::Client;
use config::Config;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[derive(Parser)]
#[command(name = "comma-cli", version, about = "CLI tools for comma devices")]
struct Cli {
    /// Increase logging verbosity (repeat for more detail: -v = info, -vv = debug, -vvv = trace)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Fetch and display the authenticated user's profile
    Me,

    /// List devices accessible to the authenticated user
    Devices,

    /// Get information about a device (uses default dongle ID if none given)
    Device {
        /// Dongle ID (optional if default is set)
        dongle_id: Option<String>,
    },

    /// Inspect routes
    Route {
        #[command(subcommand)]
        command: RouteCommands,
    },

    /// Upload files from the device
    Upload {
        #[command(subcommand)]
        command: UploadCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show the configuration file path and current contents
    Show,

    /// Set a configuration value
    Set {
        #[command(subcommand)]
        command: ConfigSetCommands,
    },
}

#[derive(Subcommand)]
enum ConfigSetCommands {
    /// Set the default dongle ID stored in the config
    Dongle {
        /// The 16-character dongle ID
        dongle_id: String,
    },

    /// Set the comma developer JWT token stored in the config
    Token {
        /// The JWT token
        token: String,
    },
}

#[derive(Subcommand)]
enum RouteCommands {
    /// List routes that have been uploaded from a device (minimally or fully)
    List {
        /// Dongle ID (optional if default is set)
        dongle_id: Option<String>,

        /// Maximum number of routes to return
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },

    /// List uploaded files for a route (signed URLs; presence indicates upload status)
    Files {
        /// Route name: full form `dongle|timestamp` or just the timestamp part
        /// (e.g. `44379c05b64fd7e3|00000144--e88a1a5266` or `00000144--e88a1a5266`)
        name: String,

        /// Dongle ID (required only when `name` does not already contain the dongle).
        /// Falls back to `COMMA_DONGLE` env var, then config default.
        #[arg(long)]
        dongle: Option<String>,
    },
}

#[derive(Subcommand)]
enum UploadCommands {
    /// Show the device's current Athena upload queue (files and progress)
    Queue {
        /// Dongle ID (optional if default is set or COMMA_DONGLE is present)
        dongle_id: Option<String>,
    },

    /// Request upload of road camera files (fcamera.hevc) for a route
    RoadCamera {
        /// Route name: full form `dongle|timestamp` or just the timestamp part
        name: String,

        /// Dongle ID (required only when `name` does not already contain the dongle)
        #[arg(long)]
        dongle: Option<String>,

        /// Only request this many segments (default: all missing)
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Request upload of wide road camera files (ecamera.hevc) for a route
    WideRoadCamera {
        /// Route name: full form `dongle|timestamp` or just the timestamp part
        name: String,

        /// Dongle ID (required only when `name` does not already contain the dongle)
        #[arg(long)]
        dongle: Option<String>,

        /// Only request this many segments (default: all missing)
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Request upload of driver camera files (dcamera.hevc) for a route
    DriverCamera {
        /// Route name: full form `dongle|timestamp` or just the timestamp part
        name: String,

        /// Dongle ID (required only when `name` does not already contain the dongle)
        #[arg(long)]
        dongle: Option<String>,

        /// Only request this many segments (default: all missing)
        #[arg(long)]
        limit: Option<u32>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let log_path = init_tracing(cli.verbose)?;
    info!(log_file = %log_path.display(), "file logging enabled");

    match cli.command {
        Commands::Config { command } => match command {
            ConfigCommands::Show => cmd_config()?,
            ConfigCommands::Set { command } => match command {
                ConfigSetCommands::Dongle { dongle_id } => cmd_set_dongle(&dongle_id)?,
                ConfigSetCommands::Token { token } => cmd_set_token(&token)?,
            },
        },
        Commands::Me => cmd_me().await?,
        Commands::Devices => cmd_devices().await?,
        Commands::Device { dongle_id } => cmd_device(dongle_id).await?,
        Commands::Route { command } => match command {
            RouteCommands::List { dongle_id, limit } => cmd_routes(dongle_id, limit).await?,
            RouteCommands::Files { name, dongle } => cmd_route_files(name, dongle).await?,
        },
        Commands::Upload { command } => match command {
            UploadCommands::Queue { dongle_id } => cmd_upload_queue(dongle_id).await?,
            UploadCommands::RoadCamera { name, dongle, limit } => {
                cmd_upload_camera(CameraKind::Road, name, dongle, limit).await?
            }
            UploadCommands::WideRoadCamera { name, dongle, limit } => {
                cmd_upload_camera(CameraKind::WideRoad, name, dongle, limit).await?
            }
            UploadCommands::DriverCamera { name, dongle, limit } => {
                cmd_upload_camera(CameraKind::Driver, name, dongle, limit).await?
            }
        },
    }

    Ok(())
}

fn state_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("comma-rs");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/state/comma-rs");
    }
    PathBuf::from(".local/state/comma-rs")
}

fn init_tracing(verbosity: u8) -> Result<PathBuf> {
    let level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let dir = state_dir();
    fs::create_dir_all(&dir)?;
    let log_path = dir.join("comma-cli.log");

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_target(false)
        .compact()
        .with_writer(std::io::stderr)
        .with_filter(EnvFilter::new(level));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_target(true)
        .with_ansi(false)
        .with_writer(file)
        .with_filter(EnvFilter::new("trace"));

    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .init();

    debug!(verbosity, level, log_file = %log_path.display(), "tracing initialized");
    Ok(log_path)
}

fn cmd_config() -> Result<()> {
    let path = Config::path()?;
    let cfg = Config::load()?;
    info!(path = %path.display(), "loaded configuration");
    println!("Config file: {}", path.display());
    println!("{:#?}", cfg);
    Ok(())
}

fn cmd_set_dongle(dongle_id: &str) -> Result<()> {
    let mut cfg = Config::load()?;
    cfg.default_dongle_id = Some(dongle_id.to_string());
    cfg.store()?;
    info!(dongle_id, path = %Config::path()?.display(), "updated default dongle ID");
    println!("Default dongle ID set to: {}", dongle_id);
    println!("Config written to: {}", Config::path()?.display());
    Ok(())
}

fn cmd_set_token(token: &str) -> Result<()> {
    let mut cfg = Config::load()?;
    cfg.token = Some(token.to_string());
    cfg.store()?;
    info!(path = %Config::path()?.display(), "updated token");
    println!("Token stored in config.");
    println!("Config written to: {}", Config::path()?.display());
    Ok(())
}

async fn cmd_me() -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    info!("fetching user profile");
    let client = Client::new(token);
    let profile = client.get_me().await?;
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}

async fn cmd_devices() -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    info!("fetching device list");
    let client = Client::new(token);
    let devices = client.get_devices().await?;
    println!("{}", serde_json::to_string_pretty(&devices)?);
    Ok(())
}

async fn cmd_device(dongle_id: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let id = dongle_id
        .or(cfg.resolve_dongle())
        .ok_or_else(|| anyhow::anyhow!("No dongle ID provided and no default set. Use `config set dongle`, set COMMA_DONGLE, or pass one."))?;
    info!(dongle_id = %id, "fetching device info");
    let client = Client::new(token);
    let device = client.get_device(&id).await?;
    println!("{}", serde_json::to_string_pretty(&device)?);
    Ok(())
}

async fn cmd_routes(dongle_id: Option<String>, limit: u32) -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let id = dongle_id
        .or(cfg.resolve_dongle())
        .ok_or_else(|| anyhow::anyhow!("No dongle ID provided and no default set. Use `config set dongle`, set COMMA_DONGLE, or pass one."))?;
    info!(dongle_id = %id, limit, "fetching uploaded routes");
    let client = Client::new(token);
    let routes = client.get_routes(&id, Some(limit), None).await?;
    println!("{}", serde_json::to_string_pretty(&routes)?);
    Ok(())
}

/// Resolve a route name that may be either full (`dongle|ts`) or short (`ts`).
fn resolve_route_name(
    name: &str,
    dongle: Option<&str>,
    default_dongle: Option<&str>,
) -> anyhow::Result<String> {
    if name.contains('|') {
        Ok(name.to_string())
    } else {
        let id = dongle
            .or(default_dongle)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Route name does not contain a dongle ID and neither --dongle, COMMA_DONGLE, nor a default dongle was provided"
                )
            })?;
        Ok(format!("{}|{}", id, name))
    }
}

async fn cmd_route_files(name: String, dongle: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let route_name = resolve_route_name(
        &name,
        dongle.as_deref(),
        cfg.resolve_dongle().as_deref(),
    )?;
    info!(route_name = %route_name, "fetching route files");
    let client = Client::new(token);
    let files = client.get_route_files(&route_name).await?;
    println!("{}", serde_json::to_string_pretty(&files)?);
    Ok(())
}

async fn cmd_upload_queue(dongle_id: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let id = dongle_id
        .or(cfg.resolve_dongle())
        .ok_or_else(|| anyhow::anyhow!("No dongle ID provided and no default set. Use `config set dongle`, set COMMA_DONGLE, or pass one."))?;
    info!(dongle_id = %id, "fetching upload queue via Athena");
    let client = Client::new(token);
    let queue = client.list_upload_queue(&id).await?;
    if queue.is_empty() {
        println!("Upload queue is empty.");
    } else {
        println!("{}", serde_json::to_string_pretty(&queue)?);
    }
    Ok(())
}

async fn cmd_upload_camera(
    kind: CameraKind,
    name: String,
    dongle: Option<String>,
    limit: Option<u32>,
) -> Result<()> {
    use comma_connect_api::UploadFileRequest;

    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let route_name = resolve_route_name(
        &name,
        dongle.as_deref(),
        cfg.resolve_dongle().as_deref(),
    )?;
    let dongle_id = route_name
        .split('|')
        .next()
        .ok_or_else(|| anyhow::anyhow!("route name missing dongle id"))?
        .to_string();
    let route_ts = route_name
        .split('|')
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("route name missing timestamp part"))?
        .to_string();

    info!(route_name = %route_name, camera = kind.display(), "requesting camera upload");
    let client = Client::new(token);

    let route = client.get_route(&route_name).await?;
    let segment_count = route
        .maxqlog
        .map(|m| m + 1)
        .ok_or_else(|| anyhow::anyhow!("route has no maxqlog; cannot determine segment count"))?;

    let files = client.get_route_files(&route_name).await?;
    let already = kind.uploaded(&files);
    info!(segment_count, already_uploaded = already.len(), "segment status");

    let mut paths = Vec::new();
    for n in 0..segment_count {
        if (n as usize) < already.len() {
            continue;
        }
        paths.push(format!("{}--{}/{}", route_ts, n, kind.filename()));
        if let Some(lim) = limit {
            if paths.len() as u32 >= lim {
                break;
            }
        }
    }

    if paths.is_empty() {
        println!(
            "All {} segments already uploaded (or none missing).",
            kind.display()
        );
        return Ok(());
    }

    println!(
        "Requesting upload for {} {} file(s)...",
        paths.len(),
        kind.display()
    );
    let urls = client.get_upload_urls(&dongle_id, &paths, Some(7)).await?;
    if urls.len() != paths.len() {
        return Err(anyhow::anyhow!(
            "upload_urls returned {} URLs for {} paths",
            urls.len(),
            paths.len()
        ));
    }

    let requests: Vec<UploadFileRequest> = paths
        .into_iter()
        .zip(urls)
        .map(|(fn_, u)| UploadFileRequest {
            file_name: fn_,
            url: u.url,
            headers: u.headers.unwrap_or_default(),
            allow_cellular: false,
            priority: 1,
        })
        .collect();

    let result = client
        .upload_files_to_urls(&dongle_id, &requests, None)
        .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Clone, Copy)]
enum CameraKind {
    Road,
    WideRoad,
    Driver,
}

impl CameraKind {
    fn filename(self) -> &'static str {
        match self {
            CameraKind::Road => "fcamera.hevc",
            CameraKind::WideRoad => "ecamera.hevc",
            CameraKind::Driver => "dcamera.hevc",
        }
    }

    fn display(self) -> &'static str {
        match self {
            CameraKind::Road => "road camera",
            CameraKind::WideRoad => "wide road camera",
            CameraKind::Driver => "driver camera",
        }
    }

    fn uploaded(self, files: &comma_connect_api::RouteFiles) -> &[String] {
        match self {
            CameraKind::Road => &files.cameras,
            CameraKind::WideRoad => &files.ecameras,
            CameraKind::Driver => &files.dcameras,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_route_name;

    #[test]
    fn full_name_is_passed_through() {
        let name = "44379c05b64fd7e3|00000144--e88a1a5266";
        let resolved = resolve_route_name(name, None, None).unwrap();
        assert_eq!(resolved, name);
    }

    #[test]
    fn short_name_uses_explicit_dongle() {
        let resolved = resolve_route_name(
            "00000144--e88a1a5266",
            Some("44379c05b64fd7e3"),
            None,
        )
        .unwrap();
        assert_eq!(resolved, "44379c05b64fd7e3|00000144--e88a1a5266");
    }

    #[test]
    fn short_name_falls_back_to_default_dongle() {
        let resolved = resolve_route_name(
            "00000144--e88a1a5266",
            None,
            Some("44379c05b64fd7e3"),
        )
        .unwrap();
        assert_eq!(resolved, "44379c05b64fd7e3|00000144--e88a1a5266");
    }

    #[test]
    fn short_name_without_dongle_errors() {
        let err = resolve_route_name("00000144--e88a1a5266", None, None).unwrap_err();
        assert!(err.to_string().contains("dongle"));
    }
}
