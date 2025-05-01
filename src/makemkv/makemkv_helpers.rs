use std::process::Output;

use super::{
    errors::{MakeMkvError, Result},
    makemkv_core::MAKE_MKV,
};
use crate::{debug, error, info, trace};

pub struct Command<'a> {
    pub command: &'a str,
    pub args: Vec<String>,
}

/// A struct representing a command to be executed, along with its arguments.
/// This struct is designed to facilitate the execution of external commands
/// asynchronously using Tokio's process handling utilities.
///
/// # Lifetime Parameters
/// - `'a`: The lifetime of the command string, which is stored as a leaked
///   boxed string to ensure it lives for the duration of the program.
///
/// # Methods
///
/// ## `new`
/// Constructs a new `Command` instance with the specified command and arguments.
///
/// ### Parameters
/// - `command`: A type that can be converted into a `String`, representing the
///   command to be executed.
/// - `args`: A vector of strings representing the arguments to be passed to the
///   command.
///
/// ### Returns
/// A `Command` instance containing the provided command and arguments.
///
/// ### Example
/// ```rust
/// let command = Command::new("ls", vec!["-la".to_string()]);
/// ```
///
/// ## `execute`
/// Executes the command asynchronously and returns the output.
///
/// ### Returns
/// - `Ok(Output)`: The output of the command if it executes successfully.
/// - `Err`: An error if the command fails to execute.
///
/// ### Behavior
/// - Logs the command and its arguments at the `trace` level before execution.
/// - Logs the command's output at the `trace` level after execution.
/// - Ensures the command process is killed if dropped before completion.
///
/// ### Example
/// ```rust
/// let command = Command::new("ls", vec!["-la".to_string()]);
/// let output = command.execute().await?;
/// println!("Command output: {:?}", output);
/// ```
///
/// # Notes
/// - The `command` field is stored as a leaked boxed string to ensure its
///   lifetime matches the `'a` lifetime parameter.
/// - This struct is designed to work with Tokio's asynchronous runtime.
impl<'a> Command<'a> {
    pub fn new<S: Into<String>>(command: S, args: Vec<String>) -> Command<'a> {
        Command {
            command: Box::leak(command.into().into_boxed_str()),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    pub async fn execute(&self) -> Result<Output> {
        trace!("Executing command: {} {:?}", self.command, self.args);
        let output = tokio::process::Command::new(&self.command)
            .args(&self.args)
            .kill_on_drop(true)
            .output()
            .await?;

        trace!("Command output: {:?}", output);
        Ok(output)
    }
}

#[derive(Default, Debug)]
/// Represents information about a disc, including its name and the titles it contains.
///
/// # Fields
/// - `disc_name`: The name of the disc as a `String`.
/// - `titles`: A vector of `Title` structs representing the titles available on the disc.
///
/// This struct is typically used to encapsulate metadata about a disc, such as its name
/// and the list of titles it contains, which can be processed or displayed by the application.
pub struct DiscInfo {
    pub disc_name: String,
    pub titles: Vec<Title>,
}

#[derive(Default, Clone, Debug)]
/// Represents a title in the MakeMKV context, containing metadata about the title.
///
/// This struct is used to store information about a specific title, such as its
/// ID, number of chapters, length, size, bitrate, resolution, aspect ratio, and frame rate.
///
/// # Fields
///
/// - `title_id` - The unique identifier for the title.
/// - `chapters` - The number of chapters in the title.
/// - `length` - The total duration of the title, typically represented as a string (e.g., "01:30:00").
/// - `size` - The size of the title, typically represented as a string (e.g., "4.7 GB").
/// - `bitrate` - The bitrate of the title, typically represented as a string (e.g., "5 Mbps").
/// - `resolution` - The resolution of the title, typically represented as a string (e.g., "1920x1080").
/// - `aspect_ratio` - The aspect ratio of the title, typically represented as a string (e.g., "16:9").
/// - `frame_rate` - The frame rate of the title, typically represented as a string (e.g., "24 fps").
///
/// This struct is useful for organizing and accessing detailed information about
/// media titles during processing or analysis.
pub struct Title {
    pub title_id: u16,
    pub chapters: u16,
    pub length: String,
    pub size: String,
    pub bitrate: String,
    pub resolution: String,
    pub aspect_ratio: String,
    pub frame_rate: String,
}

#[derive(Debug)]
/// Represents a physical or virtual drive that can be used for media ripping.
///
/// This struct contains information about a specific drive, including its
/// identifying number, model name, and the title of the media currently
/// loaded in the drive.
///
/// # Fields
///
/// * `drive_number` - A unique identifier for the drive, represented as an unsigned 8-bit integer.
/// * `drive_model` - A string representing the model name or identifier of the drive.
/// * `drive_media_title` - A string representing the title of the media currently loaded in the drive.
///
/// # Example
///
/// ```rust
/// use cord_ripper_v1::makemkv::makemkv_helpers::Drive;
///
/// let drive = Drive {
///     drive_number: 1,
///     drive_model: String::from("ASUS BW-16D1HT"),
///     drive_media_title: String::from("My Movie Disc"),
/// };
///
/// println!("Drive {}: {} with media '{}'",
///     drive.drive_number,
///     drive.drive_model,
///     drive.drive_media_title
/// );
/// ```
pub struct Drive {
    pub drive_number: u8,
    pub drive_model: String,
    pub drive_media_title: String,
}

pub async fn makemkv_exists() -> bool {
    let command = Command {
        command: "makemkvcon",
        args: vec![],
    };

    // Execite the command and check the output to see if MakeMKV responded
    let output = command.execute().await;
    match output {
        Ok(output) => {
            trace!("MakeMKV output status code: {:?}", output.status.code());
            if output.status.code() == Some(1) {
                return true;
            }
        }
        Err(_) => {}
    }

    // If the command fails or MakeMKV is not installed, return false
    // Defaults to failure
    false
}

pub fn check_makemkv_output(output: &Output) -> Result<()> {
    // Convert the output from a vec of bytes to a string
    let stdout_string = String::from_utf8(output.stdout.clone())?;

    if output.status.success() {
        // Even if the command is successful, we need to check the output
        // for any errors that may have occurred, noted by 'Failed to save'
        if stdout_string.contains("Failed to save") {
            error!("Failed to read disc! Likely a scratched or currupt disc.");
            return Err(MakeMkvError::FailedToSaveDisc);
        } else {
            return Ok(());
        }
    } else if let Some(exit_code) = output.status.code() {
        debug!("MakeMKV exited with code: {}", exit_code);
        // Specific codes reverse engineered from the MakeMKV source code
        // and other makemkv related projects
        if exit_code == 11 {
            if stdout_string.lines().count() > 3 {
                error!("Failed to open disc due to unknonwn error.");
                return Err(MakeMkvError::UnknownError);
            } else {
                error!("Failed to open disc. Please wait a moment and try again. \nIf issue persists, please cycle drive tray.");
                return Err(MakeMkvError::DriveError);
            }
        }
    }

    // If we reach this point, it means the command failed and we don't know why
    error!("Something crazy bad happened!? Please report this to the developers.");
    Err(MakeMkvError::UnknownError)
}

pub async fn get_drives() -> Result<Vec<Drive>> {
    info!("Getting data from drives...");
    let command = Command::new(
        "makemkvcon",
        vec![
            "-r".to_string(),
            "--cache=1".to_string(),
            "info".to_string(),
            "disc:9999".to_string(),
        ],
    );

    // Execute the command and check the output to see if MakeMKV responded
    let output = command.execute().await.map_err(|e| {
        error!("Failed to execute MakeMKV command: {}", e);
        MakeMkvError::CommandExecutionError(e.to_string())
    })?;

    let mut discs = Vec::new();

    // Yikes this is a clutterfuck
    // Parse the output to find the drives
    for line in String::from_utf8(output.stdout)?.lines() {
        // In combination with the 'robotic output' of makemkvcon,
        // drive information is always prefixed with 'DRV:'
        // and drive information is *always* stored the same
        if line.starts_with("DRV:") && line.contains("/dev/sr") {
            // Info is displayed in csv format
            let info: Vec<&str> = line.split(",").collect();
            let disc_no: u8 = info[6][8..9].parse()?;
            let inserted_disc = if clean_str(info[5]) == "" {
                "No disc inserted".to_string()
            } else {
                clean_str(info[5]).replace("_", " ")
            };
            let drive_info = clean_str(info[4]);

            // Wow we actually got a drive instance from this??
            discs.push(Drive {
                drive_number: disc_no + 1,
                drive_model: drive_info,
                drive_media_title: inserted_disc,
            });
        }
    }

    discs.sort_by_key(|drive| drive.drive_number);

    debug!("Found following drives: {:?}", discs);

    // Check if we found any drives, unlikely with the rats nest of references, but possible
    if discs.is_empty() {
        error!("No drives found");
        return Err(MakeMkvError::NoDrivesFound);
    }

    Ok(discs)
}

pub async fn get_title_info(drive_number: u8) -> Result<DiscInfo> {
    info!("Grabbing title info");

    // A current limitation of the current edition is that titles are limited to
    // a minimum length of 10 minutes
    // This started as a feature to filter advertisement titles, but it is now a limitation
    // but it's not going to change until there is a demand for it
    let command = Command::new(
        "makemkvcon",
        vec![
            "-r".to_string(),
            "info".to_string(),
            format!("dev:/dev/sr{}", drive_number - 1),
            "--minlength=600".to_string(),
        ],
    );

    // Execute the command and check the output to see if MakeMKV responded
    let output = command.execute().await.map_err(|e| {
        error!("Failed to execute MakeMKV command: {}", e);
        MakeMkvError::CommandExecutionError(e.to_string())
    })?;

    // Check to see if the disc info was successfully retrieved
    let disc_info = parse_disc_info(&output).map_err(|e| {
        error!("Failed to parse MakeMKV output: {}", e);
        MakeMkvError::ParseError(e.to_string())
    })?;

    // Wow... we made it, take this king: 👑
    Ok(disc_info)
}

pub fn parse_disc_info(output: &Output) -> Result<DiscInfo> {
    // Create some empty structs to store the disc and title info
    let mut disc_info = DiscInfo::default();
    let mut title_info = Title::default();

    // Iter through lines of converted utf8 bytes
    for line in String::from_utf8(output.stdout.clone())?.lines() {
        // Trim the line
        let line = line.trim();
        trace!("{}", line);
        // Create a vec of strings from the line
        let info: Vec<&str> = line.split(",").collect();

        // Check if the line starts with 'CINFO', 'TINFO', or 'SINFO'
        // and parse the info accordingly
        // A reference to the info_codes are available online in the 'apdefs.h' file
        if line.starts_with("CINFO") {
            let info_code: u8 = info[0].split(":").last().unwrap().parse()?;
            if info_code == 2 {
                disc_info.disc_name = clean_info(info);
            }
        } else if line.starts_with("TINFO") {
            let mut title_code: u8 = info[0].split(":").last().unwrap().parse()?;
            title_code = title_code + 1;

            title_info.title_id = title_code as u16;

            let info_code: u8 = info[1].parse()?;

            // Get title info
            match info_code {
                8 => title_info.chapters = clean_info(info).parse()?,
                9 => title_info.length = clean_info(info),
                10 => title_info.size = clean_info(info),
                _ => continue,
            }
        } else if line.starts_with("SINFO") {
            // I forgor how this works or if it can be used elseware
            let info_code: u8 = info[2].parse()?;

            // Get additional title info
            match info_code {
                13 => title_info.bitrate = clean_info(info),
                19 => title_info.resolution = clean_info(info),
                20 => title_info.aspect_ratio = clean_info(info),
                21 => {
                    title_info.frame_rate = clean_info(info);
                    disc_info.titles.push(title_info.clone());
                }
                _ => continue,
            }
        }
    }

    trace!("Parsed disc info: {:?}", disc_info);

    Ok(disc_info)
}

pub async fn get_last_episode_in_dir(title: &str, season: u8) -> Result<u8> {
    let mut last_episode = 0;

    let makemkv = MAKE_MKV.lock().await;

    // Logic repeated before? maybe i should extract this to a function
    let season_dir = makemkv
        .output_dir
        .join(format!("shows/{}/Season {}", title, season));
    if !season_dir.exists() {
        debug!(
            "Season directory does not exist: {}, setting to 0",
            season_dir.to_string_lossy()
        );
        return Ok(0);
    }
    let entries = std::fs::read_dir(&season_dir)
        .map_err(|_| MakeMkvError::FileNotFoundError(season_dir.to_string_lossy().to_string()))?;

    trace!("Entries in {} Season {}: {:?}", title, season, entries);

    for entry in entries {
        trace!("Entry: {:?}", entry);
        let entry = entry.map_err(|_| {
            MakeMkvError::FileNotFoundError(season_dir.to_string_lossy().to_string())
        })?;
        let path = entry.path();

        // THIS IS SO BAD LMAOOOO
        // Check if the path is a file and if it starts with "Episode " and ends with ".mkv"
        if path.is_file() {
            // IF the file_name returns something, and it can be converted to a string
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    // Check if the file name starts with "Episode " and ends with ".mkv"
                    if file_name_str.starts_with("Episode ") {
                        if let Some(episode_str) = file_name_str.split_whitespace().nth(1) {
                            if let Ok(episode) = episode_str.replace(".mkv", "").parse::<u8>() {
                                last_episode = last_episode.max(episode);
                            }
                        }
                    }
                }
            }
        }
    }

    trace!(
        "Last episode in {} Season {}: {}",
        title,
        season,
        last_episode
    );

    Ok(last_episode)
}

// Simple functioin to clean up the string
fn clean_info(info: Vec<&str>) -> String {
    clean_str(info.last().unwrap())
}

fn clean_str(s: &str) -> String {
    s.trim().replace("\"", "").to_string()
}
