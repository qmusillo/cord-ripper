use serenity::all::{
    ComponentInteractionDataKind, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, EditMessage, Interaction,
};

use serenity::builder::{CreateSelectMenuKind, CreateSelectMenuOption};

use crate::makemkv::get_title_info;

use crate::{debug, trace};

pub fn register() -> CreateCommand {
    debug!("Regisered get_titles command");
    CreateCommand::new("get_titles").description("View the available titles on the disc")
}

pub async fn run(ctx: &Context, interaction: &Interaction) {
    debug!("Running get_titles command");

    match interaction {
        Interaction::Command(command) => {
            trace!("Got request from command interaction");

            command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .add_embed(
                                CreateEmbed::new()
                                    .title("Select a disc to view titles")
                                    .description(
                                        "Please select a disc to view the available titles.",
                                    )
                                    .color(0xfe0000),
                            )
                            .select_menu(CreateSelectMenu::new(
                                "select_disc_to_grab_titles",
                                CreateSelectMenuKind::String {
                                    options: vec![
                                        CreateSelectMenuOption::new("Disc 1", "disc_1"),
                                        CreateSelectMenuOption::new("Disc 2", "disc_2"),
                                        CreateSelectMenuOption::new("Disc 3", "disc_3"),
                                    ],
                                },
                            )),
                    ),
                )
                .await
                .unwrap();
        }
        Interaction::Component(component) => {
            trace!("Got request from component interaction");

            let drive_number: u8 = match &component.data.kind {
                ComponentInteractionDataKind::StringSelect { values } => {
                    values[0].replace("disc_", "").parse().unwrap()
                }
                _ => {
                    debug!(
                        "Unknown component interaction data kind: {:?}",
                        component.data.kind
                    );
                    return;
                }
            };

            let title_info_future = get_title_info(drive_number);

            let mut message = component.message.clone();
            message
                .edit(
                    &ctx.http,
                    EditMessage::new()
                        .embed(
                            CreateEmbed::new()
                                .title(format!("Getting titles for Disc {}", drive_number))
                                .description("Please wait...")
                                .color(0xfe0000),
                        )
                        .components(vec![]),
                )
                .await
                .unwrap();

            let title_info = title_info_future.await.unwrap();

            let mut embeds = vec![CreateEmbed::new()
                .title(title_info.disc_name)
                .color(0xfe0000)
                .description(format!("Found {} titles", title_info.titles.len()))];

            let mut description = String::new();
            for title in &title_info.titles {
                description.push_str(&format!(
                    "**Title {}**\nDuration: {}\nChapters: {}\nSize: {}\nResolution: {}\nFrame Rate: {}\n\n",
                    title.title_id, title.length, title.chapters, title.size, title.resolution, title.frame_rate
                ));

                // If the description gets too long, create a new embed
                if description.len() > 1000 {
                    embeds.push(
                        CreateEmbed::new()
                            .description(description.clone())
                            .color(0xfe0000),
                    );
                    description.clear();
                }
            }

            // Add the remaining description as an embed
            if !description.is_empty() {
                embeds.push(CreateEmbed::new().description(description).color(0xfe0000));
            }

            message
                .edit(
                    &ctx.http,
                    EditMessage::new().embeds(embeds).components(vec![]),
                )
                .await
                .unwrap();
        }

        _ => {
            debug!("Unknown interaction type: {:?}, ignoring", interaction);
            return;
        }
    }
}
