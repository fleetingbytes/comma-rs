mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use comma_connect_api::Client;
use config::Config;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

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
    /// Manage the device's Athena upload queue
    Queue {
        #[command(subcommand)]
        command: QueueCommands,
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

    /// Request upload of full log files (rlog.zst) for a route
    Log {
        /// Route name: full form `dongle|timestamp` or just the timestamp part
        name: String,

        /// Dongle ID (required only when `name` does not already contain the dongle)
        #[arg(long)]
        dongle: Option<String>,

        /// Only request this many segments (default: all missing)
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Request upload of all file kinds for a route (road, driver, wide road, log),
    /// zipped by segment. Already-uploaded kinds of a segment are skipped.
    Route {
        /// Route name: full form `dongle|timestamp` or just the timestamp part
        name: String,

        /// Dongle ID (required only when `name` does not already contain the dongle)
        #[arg(long)]
        dongle: Option<String>,

        /// Only process this many segments (default: all)
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Upload all missing file kinds for recent routes.
    ///
    /// Without `--limit`, every route in the last `--max-days` (default 14) that
    /// still has missing files is enqueued. With `--limit`, only the N most
    /// recent routes in that window are considered.
    RecentRoutes {
        /// Dongle ID (optional if default is set or COMMA_DONGLE is present)
        dongle_id: Option<String>,

        /// Only consider this many of the most recent routes
        #[arg(long)]
        limit: Option<u32>,

        /// Only consider routes created within this many days
        #[arg(long, default_value_t = 14)]
        max_days: u32,
    },
}

#[derive(Subcommand)]
enum QueueCommands {
    /// List the device's current Athena upload queue (files and progress)
    List {
        /// Dongle ID (optional if default is set or COMMA_DONGLE is present)
        dongle_id: Option<String>,
    },

    /// Cancel uploads from the queue
    Cancel {
        #[command(subcommand)]
        command: CancelCommands,
    },
}

#[derive(Subcommand)]
enum CancelCommands {
    /// Cancel a single pending upload by its queue item id
    Id {
        /// The id of the queue item (from `upload queue list`)
        id: String,

        /// Dongle ID (optional if default is set or COMMA_DONGLE is present)
        dongle_id: Option<String>,
    },

    /// Cancel all pending (non-current) uploads
    Pending {
        /// Dongle ID (optional if default is set or COMMA_DONGLE is present)
        dongle_id: Option<String>,
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
            UploadCommands::Queue { command } => match command {
                QueueCommands::List { dongle_id } => cmd_upload_queue_list(dongle_id).await?,
                QueueCommands::Cancel { command } => match command {
                    CancelCommands::Id { id, dongle_id } => {
                        cmd_upload_queue_cancel_id(id, dongle_id).await?
                    }
                    CancelCommands::Pending { dongle_id } => {
                        cmd_upload_queue_cancel_pending(dongle_id).await?
                    }
                },
            },
            UploadCommands::RoadCamera {
                name,
                dongle,
                limit,
            } => cmd_upload_files(FileKind::Road, name, dongle, limit).await?,
            UploadCommands::WideRoadCamera {
                name,
                dongle,
                limit,
            } => cmd_upload_files(FileKind::WideRoad, name, dongle, limit).await?,
            UploadCommands::DriverCamera {
                name,
                dongle,
                limit,
            } => cmd_upload_files(FileKind::Driver, name, dongle, limit).await?,
            UploadCommands::Log {
                name,
                dongle,
                limit,
            } => cmd_upload_files(FileKind::Log, name, dongle, limit).await?,
            UploadCommands::Route {
                name,
                dongle,
                limit,
            } => cmd_upload_route(name, dongle, limit).await?,
            UploadCommands::RecentRoutes {
                dongle_id,
                limit,
                max_days,
            } => cmd_upload_recent_routes(dongle_id, limit, max_days).await?,
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
    let route_name = resolve_route_name(&name, dongle.as_deref(), cfg.resolve_dongle().as_deref())?;
    info!(route_name = %route_name, "fetching route files");
    let client = Client::new(token);
    let files = client.get_route_files(&route_name).await?;
    println!("{}", serde_json::to_string_pretty(&files)?);
    Ok(())
}

fn resolve_dongle(cfg: &Config, dongle_id: Option<String>) -> Result<String> {
    dongle_id
        .or(cfg.resolve_dongle())
        .ok_or_else(|| anyhow::anyhow!("No dongle ID provided and no default set. Use `config set dongle`, set COMMA_DONGLE, or pass one."))
}

async fn cmd_upload_queue_list(dongle_id: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let id = resolve_dongle(&cfg, dongle_id)?;
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

async fn cmd_upload_queue_cancel_id(item_id: String, dongle_id: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let id = resolve_dongle(&cfg, dongle_id)?;
    info!(dongle_id = %id, item_id = %item_id, "cancelling upload");
    let client = Client::new(token);
    let result = client.cancel_upload(&id, &[item_id.clone()]).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn cmd_upload_queue_cancel_pending(dongle_id: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let id = resolve_dongle(&cfg, dongle_id)?;
    info!(dongle_id = %id, "cancelling all pending uploads");
    let client = Client::new(token);
    let n = client.cancel_pending_uploads(&id).await?;
    if n == 0 {
        println!("No pending uploads to cancel.");
    } else {
        println!("Cancelled {n} pending upload(s).");
    }
    Ok(())
}

async fn cmd_upload_files(
    kind: FileKind,
    name: String,
    dongle: Option<String>,
    limit: Option<u32>,
) -> Result<()> {
    use comma_connect_api::UploadFileRequest;

    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let route_name = resolve_route_name(&name, dongle.as_deref(), cfg.resolve_dongle().as_deref())?;
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

    info!(route_name = %route_name, kind = kind.display(), "requesting upload");
    let client = Client::new(token);

    let route = client.get_route(&route_name).await?;
    let segment_count = route
        .maxqlog
        .map(|m| m + 1)
        .ok_or_else(|| anyhow::anyhow!("route has no maxqlog; cannot determine segment count"))?;

    let files = client.get_route_files(&route_name).await?;
    let already = kind.uploaded(&files);
    info!(
        segment_count,
        already_uploaded = already.len(),
        "segment status"
    );

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

/// Order used by `upload route`: for each segment, request these kinds.
/// Matches the web UI's "All Files" behavior of walking kinds per segment
/// and skipping any kind already present for that segment.
const ROUTE_UPLOAD_ORDER: [FileKind; 4] = [
    FileKind::Road,
    FileKind::Driver,
    FileKind::WideRoad,
    FileKind::Log,
];

/// Build the zipped path list for a whole-route upload.
///
/// For segment `n`, a kind is included only when `uploaded[kind].len() <= n`
/// (the `/files` arrays are ordered by segment number). `limit` caps the
/// number of segments considered, not the number of files.
fn missing_route_paths(
    route_ts: &str,
    segment_count: i32,
    files: &comma_connect_api::RouteFiles,
    limit: Option<u32>,
) -> Vec<String> {
    let segments = match limit {
        Some(l) => segment_count.min(l as i32),
        None => segment_count,
    };
    let mut paths = Vec::new();
    for n in 0..segments {
        for kind in ROUTE_UPLOAD_ORDER {
            if (n as usize) < kind.uploaded(files).len() {
                continue;
            }
            paths.push(format!("{}--{}/{}", route_ts, n, kind.filename()));
        }
    }
    paths
}

async fn cmd_upload_route(name: String, dongle: Option<String>, limit: Option<u32>) -> Result<()> {
    use comma_connect_api::UploadFileRequest;

    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let route_name = resolve_route_name(&name, dongle.as_deref(), cfg.resolve_dongle().as_deref())?;
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

    info!(route_name = %route_name, "requesting whole-route upload");
    let client = Client::new(token);

    let route = client.get_route(&route_name).await?;
    let segment_count = route
        .maxqlog
        .map(|m| m + 1)
        .ok_or_else(|| anyhow::anyhow!("route has no maxqlog; cannot determine segment count"))?;

    let files = client.get_route_files(&route_name).await?;
    let paths = missing_route_paths(&route_ts, segment_count, &files, limit);
    info!(segment_count, files = paths.len(), "whole-route plan");

    if paths.is_empty() {
        println!("All files for this route are already uploaded.");
        return Ok(());
    }

    println!(
        "Requesting upload for {} file(s) across the route...",
        paths.len()
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

/// Unix timestamp in seconds for `days` ago.
fn cutoff_secs(now_secs: u64, days: u32) -> i64 {
    now_secs.saturating_sub(days as u64 * 24 * 60 * 60) as i64
}

async fn enqueue_paths(
    client: &Client,
    dongle_id: &str,
    paths: Vec<String>,
) -> Result<serde_json::Value> {
    use comma_connect_api::UploadFileRequest;
    let urls = client.get_upload_urls(dongle_id, &paths, Some(7)).await?;
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
    Ok(client
        .upload_files_to_urls(dongle_id, &requests, None)
        .await?)
}

async fn cmd_upload_recent_routes(
    dongle_id: Option<String>,
    limit: Option<u32>,
    max_days: u32,
) -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let id = dongle_id
        .or(cfg.resolve_dongle())
        .ok_or_else(|| anyhow::anyhow!("No dongle ID provided and no default set. Use `config set dongle`, set COMMA_DONGLE, or pass one."))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let created_after = cutoff_secs(now, max_days);

    info!(dongle_id = %id, max_days, created_after, limit, "scanning recent routes");
    let client = Client::new(token);
    let routes = client.get_routes(&id, limit, Some(created_after)).await?;

    if routes.is_empty() {
        println!("No routes in the last {max_days} day(s).");
        return Ok(());
    }

    println!(
        "Considering {} route(s) from the last {max_days} day(s)...",
        routes.len()
    );

    let mut enqueued_routes = 0usize;
    let mut enqueued_files = 0usize;
    for route in &routes {
        let Some(maxqlog) = route.maxqlog else {
            info!(route = %route.fullname, "skipping route with no maxqlog");
            continue;
        };
        let segment_count = maxqlog + 1;
        let route_ts = route.fullname.split('|').nth(1).unwrap_or(&route.fullname);
        let files = client.get_route_files(&route.fullname).await?;
        let paths = missing_route_paths(route_ts, segment_count, &files, None);
        if paths.is_empty() {
            println!("  {} — complete", route.fullname);
            continue;
        }
        println!(
            "  {} — {} missing file(s), requesting upload",
            route.fullname,
            paths.len()
        );
        let result = enqueue_paths(&client, &id, paths.clone()).await?;
        let n = result
            .get("result")
            .and_then(|r| r.get("enqueued"))
            .and_then(|n| n.as_u64())
            .unwrap_or(paths.len() as u64);
        enqueued_routes += 1;
        enqueued_files += n as usize;
        info!(route = %route.fullname, enqueued = n, "route enqueued");
    }

    if enqueued_routes == 0 {
        println!("Nothing to upload.");
    } else {
        println!("Enqueued {enqueued_files} file(s) across {enqueued_routes} route(s).");
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum FileKind {
    Road,
    WideRoad,
    Driver,
    Log,
}

impl FileKind {
    fn filename(self) -> &'static str {
        match self {
            FileKind::Road => "fcamera.hevc",
            FileKind::WideRoad => "ecamera.hevc",
            FileKind::Driver => "dcamera.hevc",
            FileKind::Log => "rlog.zst",
        }
    }

    fn display(self) -> &'static str {
        match self {
            FileKind::Road => "road camera",
            FileKind::WideRoad => "wide road camera",
            FileKind::Driver => "driver camera",
            FileKind::Log => "log",
        }
    }

    fn uploaded(self, files: &comma_connect_api::RouteFiles) -> &[String] {
        match self {
            FileKind::Road => &files.cameras,
            FileKind::WideRoad => &files.ecameras,
            FileKind::Driver => &files.dcameras,
            FileKind::Log => &files.logs,
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
        let resolved =
            resolve_route_name("00000144--e88a1a5266", Some("44379c05b64fd7e3"), None).unwrap();
        assert_eq!(resolved, "44379c05b64fd7e3|00000144--e88a1a5266");
    }

    #[test]
    fn short_name_falls_back_to_default_dongle() {
        let resolved =
            resolve_route_name("00000144--e88a1a5266", None, Some("44379c05b64fd7e3")).unwrap();
        assert_eq!(resolved, "44379c05b64fd7e3|00000144--e88a1a5266");
    }

    #[test]
    fn short_name_without_dongle_errors() {
        let err = resolve_route_name("00000144--e88a1a5266", None, None).unwrap_err();
        assert!(err.to_string().contains("dongle"));
    }

    #[test]
    fn route_paths_are_zipped_by_segment_and_skip_uploaded() {
        use comma_connect_api::RouteFiles;
        let files = RouteFiles {
            cameras: vec!["a".into(), "b".into()], // segments 0,1 road done
            dcameras: vec!["a".into()],            // segment 0 driver done
            ecameras: vec![],
            logs: vec!["a".into(), "b".into()], // segments 0,1 log done
            ..Default::default()
        };
        let paths = super::missing_route_paths("ts", 2, &files, None);
        // seg 0: road skipped, driver skipped, wide, log skipped
        // seg 1: road skipped, driver, wide, log skipped
        assert_eq!(
            paths,
            vec![
                "ts--0/ecamera.hevc".to_string(),
                "ts--1/dcamera.hevc".to_string(),
                "ts--1/ecamera.hevc".to_string(),
            ]
        );
    }

    #[test]
    fn route_paths_respect_segment_limit() {
        use comma_connect_api::RouteFiles;
        let files = RouteFiles::default();
        let paths = super::missing_route_paths("ts", 5, &files, Some(1));
        assert_eq!(
            paths,
            vec![
                "ts--0/fcamera.hevc".to_string(),
                "ts--0/dcamera.hevc".to_string(),
                "ts--0/ecamera.hevc".to_string(),
                "ts--0/rlog.zst".to_string(),
            ]
        );
    }

    #[test]
    fn cutoff_is_days_ago_in_seconds() {
        // 2026-10-15 00:00:00 UTC = 1792022400
        let cutoff = super::cutoff_secs(1792022400, 14);
        assert_eq!(cutoff, 1792022400 - 14 * 24 * 60 * 60);
    }
}
