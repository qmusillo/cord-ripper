//! # Cord Ripper
//!
//! Cord Ripper is a Rust-based application designed to interact with Discord and MakeMKV.
//! It provides a bot that can handle Discord events and integrates with MakeMKV for media processing.
//!
//! ## Features
//! - Discord bot integration using the `serenity` library.
//! - Logging with configurable log levels.
//! - Command-line argument parsing using the `clap` library.
//! - MakeMKV integration for media processing.
//!
//! ## Usage
//!
//! To run the application, you need to provide the following:
//! - A valid `DISCORD_TOKEN` environment variable for the Discord bot.
//! - A valid 'GUILD_ID' environment variable for the Discord server.
//! - Command-line arguments for logging level and output directory.
//!
//! Example:
//! ```bash
//! export DISCORD_TOKEN=your_discord_token_here
//! export GUILD_ID=your_guild_id_here
//! cargo run -- --log-level debug --output-dir /path/to/output
//! ```
//!
//! ## Command-Line Arguments
//! - `--log-level` or `-l`: Optional log level (e.g., `info`, `debug`, `warn`, etc.). Defaults to `info`.
//! - `--output-dir` or `-o`: Required path to the desired output directory.
//!
//! ## Environment Variables
//! - `DISCORD_TOKEN`: The token for the Discord bot. This must be set before running the application.
//!
//! ## Logging
//! The application uses a custom logging module to manage log levels. You can specify the log level using the `--log-level` argument.
//!
//! ## Error Handling
//! - If the `DISCORD_TOKEN` environment variable is not set, the application will log an error and exit.
//! - If MakeMKV initialization fails, the application will log the error and exit.
//! - If the Discord client fails to start, the application will log the error and exit.
//!
//! ## Modules
//! - `discord`: Contains the Discord bot implementation.
//! - `logging`: Provides logging utilities.
//! - `makemkv`: Handles MakeMKV integration.

#![warn(clippy::pedantic)]

pub mod discord;
pub mod errors;
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

    // Sets the log level based on the provided argument
    // If no argument is provided, it defaults to "info"
    if let Some(log_level) = &args.log_level {
        if let Some(log_level) = logging::log_level_from_str(log_level) {
            logging::set_log_level(log_level);
        } else {
            warn!("Invalid log level provided, using default level: info");
        }
    }

    info!("Starting server, please wait...");

    // Locks the shared MakeMKV instance and initializes it
    // If initialization fails, it logs the error and exits
    crate::makemkv::makemkv_core::MAKE_MKV
        .lock()
        .await
        .init(&args.output_dir)
        .await
        .unwrap_or_else(|e| {
            error!("Error initializing MakeMKV: {:?}", e);
            std::process::exit(1);
        });

    // Retrieves the GUILD_ID from the environment variable
    // If the variable is not set or invalid, it logs the error and exits
    let discord_token = match env::var("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            error!("DISCORD_TOKEN environment variable not set, use the command 'export DISCORD_TOKEN=your_token_here'");
            std::process::exit(1);
        }
    };

    debug!("Successfully retrieved Discord token from environment variable");

    // Creates a new Discord client with the provided token
    // If the client creation fails, it logs the error and exits
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

/// Command line arguments for the application
/// - `log_level`: Optional level of logging
/// - `output_dir`: Path to the desired output directory
///
/// This struct is used to parse command line arguments using the `clap` library.
/// The `log_level` argument is optional and can be specified using the `-l` or `--log-level` flags.
/// The `output_dir` argument is required and can be specified using the `-o` or `--output-dir` flags.
#[derive(clap::Parser, Debug)]
struct CliArgs {
    /// Optional level of logging
    #[clap(short, long, help = "Level of logging [info by default]")]
    log_level: Option<String>,
    /// Path to the desired output directory
    #[clap(short, long, help = "Path to the desired output directory")]
    output_dir: String,
}
