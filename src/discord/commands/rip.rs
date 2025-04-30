use std::vec;

use serenity::all::{
    ActionRowComponent, ComponentInteractionDataKind, Context, CreateActionRow, CreateButton,
    CreateCommand, CreateInputText, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateMessage, CreateModal, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    EditMessage, InputTextStyle, Interaction, Timestamp,
};
use serenity::builder::CreateEmbed;

use crate::makemkv::{get_title_info, Rip, RipType};

use crate::{debug, error, info, trace, warn};

pub fn register() -> CreateCommand {
    debug!("Registered rip command");
    CreateCommand::new("rip").description("Rip a disc")
}

pub async fn run(ctx: &Context, interaction: &Interaction) {
    debug!("Running rip command");

    match interaction {
        Interaction::Command(command) => {
            trace!("Got request from command interaction");

            command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .components(vec![])
                            .add_embed(
                                CreateEmbed::new()
                                    .title("Start Rip")
                                    .description("Please select a disc to run rip on.")
                                    .color(0xfe0000),
                            )
                            .select_menu(CreateSelectMenu::new(
                                "select_disc_to_rip",
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

            let mut message = component.message.clone();

            match component.data.custom_id.as_str() {
                "select_disc_to_rip" => {
                    trace!("Got select_disc_to_rip component intertaction");

                    let drive_number: u8 = match &component.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => {
                            values[0].replace("disc_", "").parse().unwrap()
                        }
                        _ => {
                            warn!("Recieved invalid component data, ignoring");
                            return;
                        }
                    };

                    component.defer(&ctx.http).await.unwrap();

                    message
                        .edit(
                            &ctx.http,
                            EditMessage::new()
                                .embed(
                                    CreateEmbed::new()
                                        .title("Select a rip type")
                                        .description("Please select a rip type to start the rip.")
                                        .color(0xfe0000)
                                        .field("Disc Number", format!("{drive_number}"), false),
                                )
                                .button(
                                    CreateButton::new("movie_rip")
                                        .label("Rip Movie")
                                        .style(serenity::all::ButtonStyle::Primary),
                                )
                                .button(
                                    CreateButton::new("show_rip")
                                        .label("Rip Show")
                                        .style(serenity::all::ButtonStyle::Primary),
                                ),
                        )
                        .await
                        .unwrap();
                }
                "movie_rip" => {
                    trace!("Got movie_rip component interaction");

                    let drive_number: u8 = match message.embeds[0].fields[0].value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!("Failed to parse disc number from message, ignoring");
                            return;
                        }
                    };

                    if let Err(e) = component
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Modal(
                                CreateModal::new(
                                    "get_title_of_movie_rip",
                                    "Please enter the title of the movie",
                                )
                                .components(vec![
                                    CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Short,
                                            "Disc Number",
                                            "disc_number",
                                        )
                                        .value(drive_number.to_string())
                                        .required(true),
                                    ),
                                    CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Short,
                                            "Movie Title",
                                            "title_of_movie",
                                        )
                                        .required(true),
                                    ),
                                ]),
                            ),
                        )
                        .await
                    {
                        error!("Failed to create get_title_of_movie_rip modal: {:?}", e);
                    }
                }
                "show_rip" => {
                    trace!("Got show_rip component interaction");

                    let drive_number: u8 = match message.embeds[0].fields[0].value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!("Failed to parse disc number from message, ignoring");
                            return;
                        }
                    };

                    if let Err(e) = component
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Modal(
                                CreateModal::new(
                                    "get_title_of_show_rip",
                                    "Please enter the title & season",
                                )
                                .components(vec![
                                    CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Short,
                                            "Disc Number",
                                            "disc_number",
                                        )
                                        .value(drive_number.to_string())
                                        .required(true),
                                    ),
                                    CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Short,
                                            "Show Title",
                                            "title_of_show",
                                        )
                                        .required(true),
                                    ),
                                    CreateActionRow::InputText(
                                        CreateInputText::new(
                                            InputTextStyle::Short,
                                            "Season",
                                            "season",
                                        )
                                        .required(true),
                                    ),
                                ]),
                            ),
                        )
                        .await
                    {
                        error!("Failed to create get_title_of_show_rip modal: {:?}", e);
                    }
                }
                "select_titles_to_rip" => {
                    trace!("Got select_titles_to_rip modal");

                    component.defer(&ctx.http).await.unwrap();

                    let drive_number: u8 = match message.embeds[0].fields[1].value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!("Failed to parse disc number from message, ignoring");
                            return;
                        }
                    };

                    let title_name: String = match message.embeds[0].fields[0].value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!("Failed to parse title from message, ignoring");
                            return;
                        }
                    };

                    let season: u8 = match message.embeds[0].fields[2].value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!("Failed to parse season from message, ignoring");
                            return;
                        }
                    };

                    let selected_titles: Vec<u8> = match &component.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => {
                            values.iter().map(|value| value.parse().unwrap()).collect()
                        }
                        _ => {
                            warn!("Recieved invalid component data, ignoring");
                            return;
                        }
                    };

                    let last_episode =
                        match crate::makemkv::get_last_episode_in_dir(&title_name, season).await {
                            Ok(value) => value,
                            Err(e) => {
                                error!("Failed to get last episode in dir: {:?}", e);
                                return;
                            }
                        };

                    let rips: Vec<Rip> = selected_titles
                        .iter()
                        .enumerate()
                        .map(|(index, &title_id)| Rip {
                            title: title_name.clone(),
                            drive_number,
                            rip_type: RipType::Show {
                                season,
                                episode: last_episode + (index as u8) + 1,
                            },
                            title_id: title_id.into(),
                        })
                        .collect();

                    trace!("Created rips: {:?}", rips);

                    let now = std::time::Instant::now();

                    let num_rips = &rips.len();
                    trace!("Number of rips: {:?}", num_rips);

                    let mut was_cancelled = false;

                    for (index, rip) in rips.iter().enumerate() {
                        let episode = if let Some(episode) = rip.episode() {
                            format!("Episode {}", episode)
                        } else {
                            warn!("No episode found for rip; very strange... ignoring");
                            continue;
                        };

                        let interaction_component = message
                            .await_component_interaction(&ctx.shard)
                            .custom_ids(vec!["cancel_rip".to_string()]);

                        //let rip_future = rip.execute();

                        if let Err(e) = message
                            .clone()
                            .edit(
                                &ctx.http,
                                EditMessage::new()
                                    .components(vec![])
                                    .embed(
                                        CreateEmbed::new()
                                            .title("Rip Show")
                                            .timestamp(Timestamp::now())
                                            .description(format!(
                                                "Ripping {}, {}... \n(Rip {}/{})",
                                                rip.title,
                                                episode,
                                                index + 1,
                                                rips.len()
                                            ))
                                            .field("Title", &rip.title, true)
                                            .field("Disc Number", drive_number.to_string(), true)
                                            .field("Season", season.to_string(), true)
                                            .color(0xfe0000),
                                    )
                                    .button(
                                        CreateButton::new("cancel_rip")
                                            .label("Cancel")
                                            .style(serenity::all::ButtonStyle::Danger),
                                    ),
                            )
                            .await
                        {
                            error!("Failed to edit show rip message: {:?}", e);
                        }

                        was_cancelled = tokio::select! {
                            rip_result = rip.execute() => {
                                if let Err(e) = rip_result {
                                    error!("Failed to execute rip: {:?}", e);
                                    if let Err(e) = message
                                        .clone()
                                        .edit(
                                            &ctx.http,
                                            EditMessage::new().components(vec![])
                                            .embed(
                                                CreateEmbed::new()
                                                    .title("Rip Failed")
                                                    .timestamp(Timestamp::now())
                                                    .description("This rip failed! Please try again.")
                                                    .field("Title", &rip.title, true)
                                                    .field("Disc Number", drive_number.to_string(), true)
                                                    .field("Season", season.to_string(), true)
                                                    .color(0xfe0000),
                                            )
                                        )
                                        .await
                                    {
                                        error!("Failed to edit show rip message: {:?}", e);
                                    }
                                }
                                false

                            }
                            Some(interaction) = interaction_component.next() => {
                                debug!("Recieved canel request");
                                if let Err(e) = interaction.defer(&ctx.http).await {
                                    error!("Failed to defer cancel request: {:?}", e);
                                }

                                if let Err(e) = rip.cancel().await{
                                    error!("Failed to cancel rip: {:?}", e);
                                };

                                if let Err(e) = message
                                    .clone()
                                    .edit(
                                        &ctx.http,
                                        EditMessage::new().components(vec![])
                                        .embed(
                                            CreateEmbed::new()
                                                .title("Rip Cancelled")
                                                .timestamp(Timestamp::now())
                                                .description("Rip cancelled!")
                                                .field("Title", &rip.title, true)
                                                .field("Disc Number", drive_number.to_string(), true)
                                                .field("Season", season.to_string(), true)
                                                .color(0xfe0000)
                                                .timestamp(Timestamp::now())
                                        )
                                    )
                                    .await
                                {
                                    error!("Failed to edit show rip message: {:?}", e);
                                }
                                info!("Rip cancelled");
                                true
                            }
                        };

                        // if let Some(component) = interaction_component.await {
                        //     trace!("Cancelling rip!");
                        // } else {
                        //     trace!("No interaction component found, continuing...");
                        // }

                        // if let Err(e) = rip_future.await {
                        //     error!("Failed to execute rip: {:?}", e);
                        // }
                    }

                    if was_cancelled {
                        return;
                    }

                    let episode_range = if num_rips > &1 {
                        format!("{}-{}", last_episode + 1, last_episode + *num_rips as u8)
                    } else {
                        format!("{}", last_episode + 1)
                    };

                    let rip_time = now.elapsed().as_secs_f64() / 60.00;

                    if let Err(e) = message
                        .clone()
                        .edit(
                            &ctx.http,
                            EditMessage::new().components(vec![]).embed(
                                CreateEmbed::new()
                                    .title(format!("Ripped {}", title_name))
                                    .description("Rips completed!")
                                    .color(0xfe0000)
                                    .timestamp(Timestamp::now()),
                            ),
                        )
                        .await
                    {
                        error!("Failed to edit show rip message: {:?}", e);
                    }

                    if let Err(e) = message
                        .channel_id
                        .send_message(
                            &ctx.http,
                            CreateMessage::new()
                                .embed(
                                    CreateEmbed::new()
                                        .title("Rip Summary")
                                        .description(format!(
                                            "Finished in: {} minutes and {:.0} seconds",
                                            rip_time.floor() as u64,
                                            (rip_time.fract() * 60.0).round()
                                        ))
                                        .field("Title", &title_name, true)
                                        .field("Disc Number", drive_number.to_string(), true)
                                        .field("Season\n", season.to_string(), true)
                                        .field("Episodes", &episode_range, true)
                                        .color(0xfe0000),
                                )
                                .reference_message(&*message),
                        )
                        .await
                    {
                        error!("Failed to send show rip summary message: {:?}", e);
                    }
                }
                "select_title_to_rip" => {
                    trace!("Got select_title_to_rip modal");

                    component.defer(&ctx.http).await.unwrap();

                    let drive_number: u8 = match message.embeds[0].fields[1].value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!("Failed to parse disc number from message, ignoring");
                            return;
                        }
                    };

                    let title_name: String = match message.embeds[0].fields[0].value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!("Failed to parse title from message, ignoring");
                            return;
                        }
                    };

                    let selected_title: u8 = match &component.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => {
                            values[0].parse().unwrap()
                        }
                        _ => {
                            warn!("Recieved invalid component data, ignoring");
                            return;
                        }
                    };

                    let rip = Rip {
                        title: title_name.clone(),
                        drive_number,
                        rip_type: RipType::Movie,
                        title_id: selected_title.into(),
                    };

                    trace!("Created rip: {:?}", rip);

                    let now = std::time::Instant::now();

                    message
                        .clone()
                        .edit(
                            &ctx.http,
                            EditMessage::new()
                                .components(vec![])
                                .embed(
                                    CreateEmbed::new()
                                        .title("Rip Movie")
                                        .timestamp(Timestamp::now())
                                        .description(format!("Ripping {}...", rip.title))
                                        .field("Title", &rip.title, true)
                                        .field("Disc Number", drive_number.to_string(), true)
                                        .color(0xfe0000),
                                )
                                .button(
                                    CreateButton::new("cancel_rip")
                                        .label("Cancel")
                                        .style(serenity::all::ButtonStyle::Danger),
                                ),
                        )
                        .await
                        .unwrap();

                    let interaction_component = message
                        .await_component_interaction(&ctx.shard)
                        .custom_ids(vec!["cancel_rip".to_string()]);

                    let was_cancelled = tokio::select! {
                        rip_result = rip.execute() => {
                            if let Err(e) = rip_result {
                                error!("Failed to execute rip: {:?}", e);
                                if let Err(e) = message
                                    .clone()
                                    .edit(
                                        &ctx.http,
                                        EditMessage::new().components(vec![])
                                        .embed(
                                            CreateEmbed::new()
                                                .title("Rip Failed")
                                                .timestamp(Timestamp::now())
                                                .description("This rip failed! Please try again.")
                                                .field("Title", &rip.title, true)
                                                .field("Disc Number", drive_number.to_string(), true)
                                                .color(0xfe0000),
                                        )
                                    )
                                    .await
                                {
                                    error!("Failed to edit show rip message: {:?}", e);
                                }
                            }
                            false

                        }
                        Some(interaction) = interaction_component.next() => {
                            debug!("Recieved canel request");
                            if let Err(e) = interaction.defer(&ctx.http).await {
                                error!("Failed to defer cancel request: {:?}", e);
                            }

                            if let Err(e) = rip.cancel().await{
                                error!("Failed to cancel rip: {:?}", e);
                            };

                            if let Err(e) = message
                                .clone()
                                .edit(
                                    &ctx.http,
                                    EditMessage::new().components(vec![])
                                    .embed(
                                        CreateEmbed::new()
                                            .title("Rip Cancelled")
                                            .timestamp(Timestamp::now())
                                            .description("Rip cancelled!")
                                            .field("Title", &rip.title, true)
                                            .field("Disc Number", drive_number.to_string(), true)
                                            .color(0xfe0000)
                                            .timestamp(Timestamp::now())
                                    )
                                )
                                .await
                            {
                                error!("Failed to edit show rip message: {:?}", e);
                            }
                            info!("Rip cancelled");
                            true
                        }
                    };

                    // let rip_future = rip.execute();

                    // if let Err(e) = rip_future.await {
                    //     warn!("Failed to execute rip: {:?}", e);
                    // }

                    if was_cancelled {
                        return;
                    }

                    let rip_time = now.elapsed().as_secs_f64() / 60.00;

                    message
                        .clone()
                        .edit(
                            &ctx.http,
                            EditMessage::new().components(vec![]).embed(
                                CreateEmbed::new()
                                    .title(format!("Ripped {}", title_name))
                                    .description("Rip completed!")
                                    .color(0xfe0000)
                                    .timestamp(Timestamp::now()),
                            ),
                        )
                        .await
                        .unwrap();

                    message
                        .channel_id
                        .send_message(
                            &ctx.http,
                            CreateMessage::new()
                                .embed(
                                    CreateEmbed::new()
                                        .title("Rip Summary")
                                        .description(format!(
                                            "Finished in: {} minutes and {:.0} seconds",
                                            rip_time.floor() as u64,
                                            (rip_time.fract() * 60.0).round()
                                        ))
                                        .field("Title", &title_name, true)
                                        .field("Disc Number", drive_number.to_string(), true)
                                        .color(0xfe0000),
                                )
                                .reference_message(&*message),
                        )
                        .await
                        .unwrap();
                }
                _ => {
                    debug!(
                        "Unknown component calling rip: {}, ignoring",
                        component.data.custom_id
                    );
                    return;
                }
            }
        }
        Interaction::Modal(modal) => {
            trace!("Got request from modal interaction");

            let message = if let Some(message) = modal.message.clone() {
                message
            } else {
                trace!("Modal interaction has no message, ignoring");
                return;
            };

            match modal.data.custom_id.as_str() {
                "get_title_of_movie_rip" => {
                    modal.defer(&ctx.http).await.unwrap();

                    let drive_number: u8 = match modal.data.components[0].components[0] {
                        ActionRowComponent::InputText(ref input) => {
                            if let Some(value) = &input.value {
                                value.parse().unwrap()
                            } else {
                                debug!("No value found for disc number, ignoring");
                                return;
                            }
                        }
                        _ => {
                            warn!("Failed to parse disc number from modal, ignoring");
                            return;
                        }
                    };

                    let title = match modal.data.components[1].components[0] {
                        ActionRowComponent::InputText(ref input) => {
                            if let Some(value) = &input.value {
                                value.clone()
                            } else {
                                debug!("No value found for title, ignoring");
                                return;
                            }
                        }
                        _ => {
                            warn!("Failed to parse title from modal, ignoring");
                            return;
                        }
                    };

                    let titles_future = get_title_info(drive_number);

                    message
                        .clone()
                        .edit(
                            &ctx.http,
                            EditMessage::new().components(vec![]).embed(
                                CreateEmbed::new()
                                    .title("Rip Movie")
                                    .description("Please wait while titles are loaded...")
                                    .field("Title", &title, true)
                                    .field("Disc Number", drive_number.to_string(), true)
                                    .color(0xfe0000),
                            ),
                        )
                        .await
                        .unwrap();

                    let titles = titles_future.await.unwrap().titles;

                    let options: Vec<CreateSelectMenuOption> = titles
                        .iter()
                        .map(|title| {
                            let title_details =
                                format!("Title: {}, Duration: {}", title.title_id, title.length);
                            let description = format!(
                                "Chapters: {}, Size: {}, Resolution: {}, Frame Rate: {}",
                                title.chapters, title.size, title.resolution, title.frame_rate
                            );
                            CreateSelectMenuOption::new(title_details, title.title_id.to_string())
                                .description(description)
                        })
                        .collect();

                    trace!("Got options: {:?}", options);

                    message
                        .clone()
                        .edit(
                            &ctx.http,
                            EditMessage::new()
                                .components(vec![CreateActionRow::SelectMenu(
                                    CreateSelectMenu::new(
                                        "select_title_to_rip",
                                        CreateSelectMenuKind::String { options },
                                    ),
                                )])
                                .embed(
                                    CreateEmbed::new()
                                        .title("Rip Movie")
                                        .description("Please select title to rip")
                                        .field("Title", &title, true)
                                        .field("Disc Number", drive_number.to_string(), true)
                                        .color(0xfe0000),
                                ),
                        )
                        .await
                        .unwrap();
                }
                "get_title_of_show_rip" => {
                    modal.defer(&ctx.http).await.unwrap();

                    let drive_number: u8 = match modal.data.components[0].components[0] {
                        ActionRowComponent::InputText(ref input) => {
                            if let Some(value) = &input.value {
                                value.parse().unwrap()
                            } else {
                                debug!("No value found for disc number, ignoring");
                                return;
                            }
                        }
                        _ => {
                            warn!("Failed to parse disc number from modal, ignoring");
                            return;
                        }
                    };

                    let title = match modal.data.components[1].components[0] {
                        ActionRowComponent::InputText(ref input) => {
                            if let Some(value) = &input.value {
                                value.clone()
                            } else {
                                warn!("No value found for title, ignoring");
                                return;
                            }
                        }
                        _ => {
                            warn!("Failed to parse title from modal, ignoring");
                            return;
                        }
                    };

                    let season = match modal.data.components[2].components[0] {
                        ActionRowComponent::InputText(ref input) => {
                            if let Some(value) = &input.value {
                                value.clone()
                            } else {
                                warn!("No value found for season, ignoring");
                                return;
                            }
                        }
                        _ => {
                            warn!("Failed to parse season from modal, ignoring");
                            return;
                        }
                    };

                    let titles_future = get_title_info(drive_number);

                    message
                        .clone()
                        .edit(
                            &ctx.http,
                            EditMessage::new().components(vec![]).embed(
                                CreateEmbed::new()
                                    .title("Rip Show")
                                    .description("Please wait while titles are loaded...")
                                    .field("Title", &title, true)
                                    .field("Disc Number", drive_number.to_string(), true)
                                    .field("Season", &season, true)
                                    .color(0xfe0000),
                            ),
                        )
                        .await
                        .unwrap();

                    let titles = titles_future.await.unwrap().titles;

                    let options: Vec<CreateSelectMenuOption> = titles
                        .iter()
                        .map(|title| {
                            let title_details =
                                format!("Title: {}, Duration: {}", title.title_id, title.length);
                            let description = format!(
                                "Chapters: {}, Size: {}, Resolution: {}, Frame Rate: {}",
                                title.chapters, title.size, title.resolution, title.frame_rate
                            );
                            CreateSelectMenuOption::new(title_details, title.title_id.to_string())
                                .description(description)
                        })
                        .collect();

                    trace!("Got options: {:?}", options);

                    let max_values = options.len() as u8;

                    trace!("Max values: {}", max_values);

                    message
                        .clone()
                        .edit(
                            &ctx.http,
                            EditMessage::new()
                                .components(vec![CreateActionRow::SelectMenu(
                                    CreateSelectMenu::new(
                                        "select_titles_to_rip",
                                        CreateSelectMenuKind::String { options },
                                    )
                                    .min_values(1)
                                    .max_values(max_values),
                                )])
                                .embed(
                                    CreateEmbed::new()
                                        .title("Rip Show")
                                        .description("Please select titles to rip")
                                        .field("Title", &title, true)
                                        .field("Disc Number", drive_number.to_string(), true)
                                        .field("Season", season, true)
                                        .color(0xfe0000),
                                ),
                        )
                        .await
                        .unwrap();
                }
                _ => {
                    debug!(
                        "Unknown modal calling rip: {}, ignoring",
                        modal.data.custom_id
                    );
                    return;
                }
            }
        }
        _ => {
            debug!("Unknown interaction type: {:?} ignoring", interaction);
            return;
        }
    }
}
