mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use comma_connect_api::Client;
use config::Config;

#[derive(Parser)]
#[command(name = "comma-cli", version, about = "CLI tools for comma devices")]
struct Cli {
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config => cmd_config()?,
        Commands::SetDongle { dongle_id } => cmd_set_dongle(&dongle_id)?,
        Commands::Me => cmd_me().await?,
        Commands::Devices => cmd_devices().await?,
        Commands::Device { dongle_id } => cmd_device(dongle_id).await?,
    }

    Ok(())
}

fn cmd_config() -> Result<()> {
    let path = Config::path()?;
    let cfg = Config::load()?;
    println!("Config file: {}", path.display());
    println!("{:#?}", cfg);
    Ok(())
}

fn cmd_set_dongle(dongle_id: &str) -> Result<()> {
    let mut cfg = Config::load()?;
    cfg.default_dongle_id = Some(dongle_id.to_string());
    cfg.store()?;
    println!("Default dongle ID set to: {}", dongle_id);
    println!("Config written to: {}", Config::path()?.display());
    Ok(())
}

async fn cmd_me() -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
    let client = Client::new(token);
    let profile = client.get_me().await?;
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}

async fn cmd_devices() -> Result<()> {
    let cfg = Config::load()?;
    let token = cfg.resolve_token()?;
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
    let client = Client::new(token);
    let device = client.get_device(&id).await?;
    println!("{}", serde_json::to_string_pretty(&device)?);
    Ok(())
}
