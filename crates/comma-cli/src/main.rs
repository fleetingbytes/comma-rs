mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use comma_connect_api::Client;
use config::Config;
use tracing::{debug, info};

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
    /// Show the configuration file path and current contents
    Config,

    /// Set the default dongle ID stored in the config
    SetDongle {
        /// The 16-character dongle ID
        dongle_id: String,
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

    /// List routes that have been uploaded from a device (minimally or fully)
    Routes {
        /// Dongle ID (optional if default is set)
        dongle_id: Option<String>,

        /// Maximum number of routes to return
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command {
        Commands::Config => cmd_config()?,
        Commands::SetDongle { dongle_id } => cmd_set_dongle(&dongle_id)?,
        Commands::Me => cmd_me().await?,
        Commands::Devices => cmd_devices().await?,
        Commands::Device { dongle_id } => cmd_device(dongle_id).await?,
        Commands::Routes { dongle_id, limit } => cmd_routes(dongle_id, limit).await?,
    }

    Ok(())
}

fn init_tracing(verbosity: u8) {
    let level = match verbosity {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_target(false)
        .compact()
        .with_writer(std::io::stderr)
        .init();

    debug!(verbosity, level = %level, "tracing initialized");
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
        .or(cfg.default_dongle_id)
        .ok_or_else(|| anyhow::anyhow!("No dongle ID provided and no default set. Use `set-dongle` or pass one."))?;
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
        .or(cfg.default_dongle_id)
        .ok_or_else(|| anyhow::anyhow!("No dongle ID provided and no default set. Use `set-dongle` or pass one."))?;
    info!(dongle_id = %id, limit, "fetching uploaded routes");
    let client = Client::new(token);
    let routes = client.get_routes(&id, Some(limit), None).await?;
    println!("{}", serde_json::to_string_pretty(&routes)?);
    Ok(())
}
