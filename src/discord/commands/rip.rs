use std::vec;

use serenity::all::{
    ActionRowComponent, ComponentInteractionDataKind, Context, CreateActionRow, CreateButton,
    CreateCommand, CreateInputText, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateMessage, CreateModal, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    EditInteractionResponse, EditMessage, InputTextStyle, Interaction, Timestamp,
};
use serenity::builder::CreateEmbed;

use crate::makemkv::{get_drives, get_title_info, Rip, RipType};

use crate::{debug, error, info, trace, warn};

pub fn register() -> CreateCommand {
    debug!("Registered rip command");
    CreateCommand::new("rip").description("Rip a disc")
}

// Wow this is gonna be the biggest roller coater of a function yet!
/// Runs the rip command
pub async fn run(ctx: &Context, interaction: &Interaction) {
    debug!("Running rip command");

    // Match the interaction type to it's associated sub function based on
    // the unique interaction id
    match interaction {
        // The initial command will be handled here, prompting the user to select a disc
        Interaction::Command(command) => {
            trace!("Got request from command interaction");

            // Satisfy discord interaction with a temperary loading message
            command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .components(vec![])
                            .embed(
                                CreateEmbed::new()
                                    .title("Loading Discs")
                                    .description("This may take a few seconds...")
                                    .color(0xfe0000),
                            ),
                    ),
                )
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to create response: {:?}", e);
                });

            // Get the drives from the makemkv library
            let drives = match get_drives().await {
                Ok(drives) => drives,
                Err(e) => {
                    error!("Failed to get drives: {:?}", e);

                    // If we fail to get the drives, edit the response to show an error message
                    if let Err(e) = command
                        .edit_response(
                            &ctx.http,
                            EditInteractionResponse::new().embed(
                                CreateEmbed::new()
                                    .title("Error")
                                    .description(
                                        "Failed to retrieve drives. Please try again later.",
                                    )
                                    .color(0xfe0000),
                            ),
                        )
                        .await
                    {
                        error!("Failed to edit response: {:?}", e);
                        return;
                    }
                    return;
                }
            };

            // Iter over the drives and create a 'select menu option' for each drive
            let options: Vec<CreateSelectMenuOption> = drives
                .iter()
                .map(|drive| {
                    CreateSelectMenuOption::new(
                        format!("Disc {}: {}", drive.drive_number, drive.drive_media_title),
                        format!("disc_{}", drive.drive_number),
                    )
                })
                .collect();

            // Create a select menu with the options
            // When the disc is selected, it will call the select_disc_to_rip component
            // interaction
            if let Err(e) = command
                .edit_response(
                    &ctx.http,
                    EditInteractionResponse::new()
                        .components(vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
                            "select_disc_to_rip",
                            CreateSelectMenuKind::String { options },
                        ))])
                        .add_embed(
                            CreateEmbed::new()
                                .title("Select Disc")
                                .description("Please select a disc to run rip on.")
                                .color(0xfe0000),
                        ),
                )
                .await
            {
                error!("Failed to edit response: {:?}", e);
                return;
            }
        }
        // Component Interactions will come from any of the buttons or select menus
        Interaction::Component(component) => {
            trace!("Got request from component interaction");

            // Satify rust borrow checker and make it easier to call
            let mut message = component.message.clone();

            // We check what type of component interaction it is by its unique id
            match component.data.custom_id.as_str() {
                // This would be recieved by the initial interaction from the command
                "select_disc_to_rip" => {
                    trace!("Got select_disc_to_rip component intertaction");

                    // Get the drive number from the component data
                    let drive_number: u8 = match &component.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => {
                            values[0].replace("disc_", "").parse().unwrap()
                        }
                        _ => {
                            warn!("Recieved invalid component data, ignoring");
                            return;
                        }
                    };

                    // Satify the interaction with a loading message
                    component.defer(&ctx.http).await.unwrap();

                    // Creates and embed to select which type of rip will be running
                    // The user will select either a movie or show rip
                    // This will split off into their respecive component interaction ids
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
                                    // This will call the movie_rip component interaction
                                    // Prompting the user to input a title
                                    // Will attempt to auto grab from the disc in the future
                                    CreateButton::new("movie_rip")
                                        .label("Rip Movie")
                                        .style(serenity::all::ButtonStyle::Primary),
                                )
                                .button(
                                    // This will call the show_rip component interaction
                                    // Prompting the user to input a title and season
                                    // Will attempt to auto grab from the disc in the future
                                    CreateButton::new("show_rip")
                                        .label("Rip Show")
                                        .style(serenity::all::ButtonStyle::Primary),
                                ),
                        )
                        .await
                        .unwrap();
                }
                // This will be called when the user selects that they want to rip a movie
                "movie_rip" => {
                    trace!("Got movie_rip component interaction");

                    // Grabs the disc number from the message embed and parses it
                    // This logic will hopefully be moved to its own function in the future
                    // Taking slices of the messege embed should be safe due to its constant
                    // positioning provided by the previous component interaction
                    let drive_number: u8 = match message.embeds[0].fields[0].value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!("Failed to parse disc number from message, ignoring");
                            return;
                        }
                    };

                    // Creates the modal for the user to input the title of the movie
                    if let Err(e) = component
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Modal(
                                // Once a title is input and the modal is submmited
                                // it will call the get_title_of_movie_rip modal interaction
                                // This will then lead to prompting the user to select
                                // a title to rip
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
                // This will be called when the user selects that they want to rip a show
                "show_rip" => {
                    trace!("Got show_rip component interaction");

                    // Repeated code I had talked about in the rip_movie component
                    let drive_number: u8 = match message.embeds[0].fields[0].value.parse() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!("Failed to parse disc number from message, ignoring");
                            return;
                        }
                    };

                    // Creates the modal for the user to input the title and season of the show
                    if let Err(e) = component
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Modal(
                                // Once title and season are input and the modal is submmited
                                // it will call the get_title_of_show_rip modal interaction
                                // This will then lead to prompting the user to select
                                // titles to rip
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
                // This will be called when the user inputs a title and season
                // for a show rip
                "select_titles_to_rip" => {
                    trace!("Got select_titles_to_rip modal");

                    // Satify the interaction
                    component.defer(&ctx.http).await.unwrap();

                    // The next 3 statements are the same as the previous component
                    // interactions, but for title_name and season as well
                    // This should be safe due to the constant positioning of the
                    // message embed fields
                    // These should be moved to their own function in the future
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

                    // Get the selected titles from the component data
                    // This will be a vector of u8s, which are the title ids
                    // This will be used to create the rips
                    let selected_titles: Vec<u8> = match &component.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => {
                            values.iter().map(|value| value.parse().unwrap()).collect()
                        }
                        _ => {
                            warn!("Recieved invalid component data, ignoring");
                            return;
                        }
                    };

                    // Gets the last episode in the directory for the show,
                    // this will be used to determine the episode number for the rip
                    let last_episode =
                        match crate::makemkv::get_last_episode_in_dir(&title_name, season).await {
                            Ok(value) => value,
                            Err(e) => {
                                error!("Failed to get last episode in dir: {:?}", e);
                                return;
                            }
                        };

                    // Iteractes over the selected titles and creates a rip for each one
                    // This will be a vector of rips, which will be used to execute the
                    // rips in sequence without requiring user input
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

                    // Satifies rust lifetime issues
                    let mut was_cancelled = false;

                    // Run the rips in sequence, updating the message with the current rip
                    // and allowing the user to cancel the rip
                    // This will be a loop that will run until all rips are complete
                    // or the user cancels the rip
                    for (index, rip) in rips.iter().enumerate() {
                        // This should only fail if the rip details are invalid and also
                        // passed previous validation
                        let episode = if let Some(episode) = rip.episode() {
                            format!("Episode {}", episode)
                        } else {
                            warn!("No episode found for rip; very strange... ignoring");
                            continue;
                        };

                        // An async handle to a 'Collector' that will be used to
                        // collect a cancel request from the user
                        // This will be used to cancel the rip if the user requests it
                        // This will be a future that will be awaited later
                        let interaction_component = message
                            .await_component_interaction(&ctx.shard)
                            .custom_ids(vec!["cancel_rip".to_string()]);

                        // Edit the message to show the current rip details
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
                                        // Add a cancel button to the message
                                        CreateButton::new("cancel_rip")
                                            .label("Cancel")
                                            .style(serenity::all::ButtonStyle::Danger),
                                    ),
                            )
                            .await
                        {
                            error!("Failed to edit show rip message: {:?}", e);
                        }

                        // The 'magic sauce' to the interaction collector
                        // tokio::select! will wait for either the rip to complete
                        // or the user to cancel the rip by waiting for either to
                        // reslove first
                        // The other statement will be cancelled
                        // sets the 'was_cancelled' variable to true if the user cancels
                        // the rip
                        was_cancelled = tokio::select! {
                            // Starts the rip and waits for it to complete
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
                            // Calls on the 'next()' method to asyncronously wait for
                            // the user to cancel the rip
                            Some(interaction) = interaction_component.next() => {
                                debug!("Recieved canel request");

                                // Defer the interaction to satify discord
                                if let Err(e) = interaction.defer(&ctx.http).await {
                                    error!("Failed to defer cancel request: {:?}", e);
                                }

                                if let Err(e) = rip.cancel().await{
                                    error!("Failed to cancel rip: {:?}", e);
                                };

                                // Edit the message to show that the rip was cancelled
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

                        // Breaks out of rip loop if the user cancels the rip
                        if was_cancelled {
                            break;
                        }
                    }

                    // If the rip was cancelled, do not send the summary message
                    if was_cancelled {
                        return;
                    }

                    // Format the episode range for the summary message
                    let episode_range = if num_rips > &1 {
                        format!("{}-{}", last_episode + 1, last_episode + *num_rips as u8)
                    } else {
                        format!("{}", last_episode + 1)
                    };

                    let rip_time = now.elapsed().as_secs_f64() / 60.00;

                    // Edit the message to show that the rip was completed
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

                    // Send a summary message to the channel with the rip details
                    // This will send a push notification to the user
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
                // This will be called when the user inputs a title
                // for a movie rip
                "select_title_to_rip" => {
                    trace!("Got select_title_to_rip modal");

                    // Satify the interaction
                    component.defer(&ctx.http).await.unwrap();

                    // Same 'needs to be extracted' code as the previous component interactions
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

                    // Only creates one rip for a movie
                    let rip = Rip {
                        title: title_name.clone(),
                        drive_number,
                        rip_type: RipType::Movie,
                        title_id: selected_title.into(),
                    };

                    trace!("Created rip: {:?}", rip);

                    let now = std::time::Instant::now();

                    // Sends a loading message to the user
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

                    // This is the same magic sauce from the show rip
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

                    // If the rip was cancelled, do not send the summary message
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
        // This would be called when the user inputs a title for a movie
        // or a title and season for a show
        Interaction::Modal(modal) => {
            trace!("Got request from modal interaction");

            // Ensires there was a message attached to the modal, otherwise disregard the interaction
            let message = if let Some(message) = modal.message.clone() {
                message
            } else {
                trace!("Modal interaction has no message, ignoring");
                return;
            };

            // Match on the modal custom id to determine which modal was called
            match modal.data.custom_id.as_str() {
                // This will be called when the user inputs a title for a movie rip
                "get_title_of_movie_rip" => {
                    // Satify the interaction
                    modal.defer(&ctx.http).await.unwrap();

                    // Some more stupid parse stuff, just now matching for ther
                    // Action row component type as well
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

                    // Starts the process of getting the title info from makemkv
                    let titles_future = get_title_info(drive_number);

                    // Sends a loading message to the user
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

                    // Awaits the title info from makemkv
                    let titles = titles_future.await.unwrap().titles;

                    // Creates a vector of select menu options from the title info
                    // This will be used to create the select menu for the user to select
                    // the title to rip
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

                    // Spawns the select menu for the user to select the title to rip
                    // This will be a single select menu, so the max values is 1
                    message
                        .clone()
                        .edit(
                            &ctx.http,
                            EditMessage::new()
                                .components(vec![CreateActionRow::SelectMenu(
                                    // Will call the select_title_to_rip component
                                    // when the user selects a title
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
                // This will be called when the user inputs a title and season for a show rip
                "get_title_of_show_rip" => {
                    // Satify the interaction
                    modal.defer(&ctx.http).await.unwrap();

                    // You know the drill, same as the previous modal just with more
                    // ... *seasoning*
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

                    // Creates a vector of select menu options from the title info
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

                    // Spawns the select menu for the user to select multiple titles to rip
                    // This will be a multi select menu, so the max values is the number of titles
                    message
                        .clone()
                        .edit(
                            &ctx.http,
                            EditMessage::new()
                                .components(vec![CreateActionRow::SelectMenu(
                                    // Will call the select_titles_to_rip component
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
