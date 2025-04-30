use serenity::all::{
    Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Interaction,
};

use crate::makemkv::get_drives;

use crate::{debug, trace};

pub fn register() -> CreateCommand {
    debug!("Regisered view_drives command");
    CreateCommand::new("view_drives").description("View the drives on the server")
}

pub async fn run(ctx: &Context, interaction: &Interaction) {
    debug!("Running view_drives command");

    let drives = get_drives().await.unwrap();

    match interaction {
        Interaction::Command(command) => {
            let mut fields = Vec::new();

            for drive in drives {
                let title = if drive.drive_media_title.is_empty() {
                    "No disc inserted".to_string()
                } else {
                    format!("Title: {}", drive.drive_media_title)
                };
                fields.push((
                    format!("Drive {}: {}", drive.drive_number, drive.drive_model),
                    title,
                    false,
                ));
            }

            command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::default().add_embed(
                            CreateEmbed::default()
                                .title("Available Drives")
                                .description("Here are the drives available on the server:")
                                .color(0xfe0000)
                                .fields(fields),
                        ),
                    ),
                )
                .await
                .unwrap();
        }
        _ => {
            debug!("Unknown interaction type: {:?}, ignoring", interaction);
            return;
        }
    }
    trace!("View drives command executed successfully");
}
