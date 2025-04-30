#![warn(clippy::pedantic)]

pub mod discord;
pub mod logging;
pub mod makemkv;

pub use logging::{current_log_level, DEBUG, ERROR, INFO, TRACE, WARN};

use clap::Parser;
use tokio;

use discord::bot::bot_core::DiscordHandler;

use serenity::prelude::{Client, GatewayIntents};

use std::env;

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    if let Some(log_level) = &args.log_level {
        if let Some(log_level) = logging::log_level_from_str(log_level) {
            logging::set_log_level(log_level);
        } else {
            warn!("Invalid log level provided, using default level: info");
        }
    }

    info!("Starting server, please wait...");

    crate::makemkv::makemkv_core::MAKE_MKV
        .lock()
        .await
        .init(&args.output_dir)
        .await
        .unwrap_or_else(|e| {
            error!("Error initializing MakeMKV: {:?}", e);
            std::process::exit(1);
        });

    let discord_token = match env::var("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            error!("DISCORD_TOKEN environment variable not set, use the command 'export DISCORD_TOKEN=your_token_here'");
            std::process::exit(1);
        }
    };
    debug!("Successfully retrieved Discord token from environment variable");

    let mut client = Client::builder(discord_token, GatewayIntents::empty())
        .event_handler(DiscordHandler)
        .await
        .unwrap_or_else(|e| {
            error!("Error creating client: {:?}", e);
            std::process::exit(1);
        });

    client.start().await.unwrap_or_else(|e| {
        error!("Error starting client: {:?}", e);
        std::process::exit(1);
    });
}

#[derive(clap::Parser, Debug)]
struct CliArgs {
    /// Optional level of logging
    #[clap(short, long, help = "Level of logging [info by default]")]
    log_level: Option<String>,
    /// Path to the desired output directory
    #[clap(short, long, help = "Path to the desired output directory")]
    output_dir: String,
}
