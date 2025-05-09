pub struct DiscordHandler;

use std::env;

use serenity::all::GuildId;
use serenity::async_trait;
use serenity::model::{application::Interaction, gateway::Ready};
use serenity::prelude::*;

use crate::discord::errors::DiscordError;
use crate::discord::{commands, errors::Result};
use crate::{debug, error, info, trace};

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Err(err) = handle_interaction(&ctx, &interaction).await {
            error!("Error handling interaction: {:?}", err);
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        let guild = match env::var("GUILD_ID") {
            Ok(guild) => match guild.parse::<u64>() {
                Ok(guild) => guild,
                Err(_) => {
                    error!("Invalid GUILD_ID provided, please provide a valid ID");
                    std::process::exit(1);
                }
            },
            Err(_) => {
                error!("GUILD_ID environment variable not set, use the command 'export GUILD_ID=your_guild_id_here'");
                std::process::exit(1);
            }
        };

        let guild_id = GuildId::new(guild);

        let commands = guild_id
            .set_commands(
                &ctx.http,
                vec![
                    commands::rip::register(),
                    commands::view_drives::register(),
                    commands::eject_disc::register(),
                    commands::get_titles::register(),
                ],
            )
            .await;

        trace!("Server now has the following guild slash commands: {commands:#?}");
        info!("The Discord bot has initialized successfully!");
        info!("Server is running...");
    }
}

pub async fn handle_interaction(ctx: &Context, interaction: &Interaction) -> Result<()> {
    trace!("Received interaction: {:?}", interaction);
    match interaction {
        Interaction::Command(command) => match command.data.name.as_str() {
            "rip" => {
                trace!("Got rip command");
                commands::rip::run(ctx, interaction).await?;
                Ok(())
            }
            "view_drives" => {
                trace!("Got view_drives command");
                commands::view_drives::run(ctx, interaction).await;
                Ok(())
            }
            "eject_disc" => {
                trace!("Got eject_disc command");
                commands::eject_disc::run();
                Ok(())
            }
            "get_titles" => {
                trace!("Got get_titles command");
                commands::get_titles::run(ctx, interaction).await;
                Ok(())
            }
            _ => {
                debug!("Unknown command: {}, ignoring", command.data.name);
                return Err(DiscordError::InvalidInteractionCall);
            }
        },
        Interaction::Component(component) => match component.data.custom_id.as_str() {
            "select_disc_to_grab_titles" => {
                trace!("Got select_disc_to_grab_titles component");
                commands::get_titles::run(ctx, interaction).await;
                Ok(())
            }
            "select_disc_to_rip" => {
                trace!("Got select_disc_to_rip component");
                commands::rip::run(ctx, interaction).await?;
                Ok(())
            }
            "movie_rip" => {
                trace!("Got movie_rip component");
                commands::rip::run(ctx, interaction).await?;
                Ok(())
            }
            "show_rip" => {
                trace!("Got show_rip component");
                commands::rip::run(ctx, interaction).await?;
                Ok(())
            }
            "select_titles_to_rip" => {
                trace!("Got select_titles_to_rip component");
                commands::rip::run(ctx, interaction).await?;
                Ok(())
            }
            "select_title_to_rip" => {
                trace!("Got select_title_to_rip component");
                commands::rip::run(ctx, interaction).await?;
                Ok(())
            }
            "cancel_rip" => {
                trace!("Got cancel_rip component");
                Ok(())
            }
            _ => {
                debug!("Unknown component: {}, ignoring", component.data.custom_id);
                Ok(())
            }
        },
        Interaction::Modal(modal) => {
            match modal.data.custom_id.as_str() {
                "get_title_of_movie_rip" => {
                    trace!("Got get_title_of_movie_rip modal");
                    commands::rip::run(ctx, interaction).await?;
                }
                "get_title_of_show_rip" => {
                    trace!("Got get_title_of_show_rip modal");
                    commands::rip::run(ctx, interaction).await?;
                }
                _ => {
                    debug!("Unknown modal: {}, ignoring", modal.data.custom_id);
                    return Err(DiscordError::InvalidInteractionCall);
                }
            }
            Ok(())
        }
        _ => {
            debug!("Unknown interaction type: {:?}, ignoring", interaction);
            return Err(DiscordError::InvalidInteractionCall);
        }
    }
}
