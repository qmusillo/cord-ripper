//! # MakeMKV Core Module
//!
//! This module provides the core functionality for interacting with the MakeMKV software
//! to rip media from optical drives. It includes abstractions for managing ripping operations,
//! handling drive locking, and organizing ripped media into appropriate directories.
//!
//! ## Overview
//!
//! The module defines the following key components:
//!
//! - **`Rip`**: Represents a ripping operation, which can either be for a movie or a specific
//!   episode of a TV show. It encapsulates metadata about the rip and provides methods to
//!   execute the ripping process asynchronously.
//!
//! - **`RipType`**: An enum that distinguishes between ripping a movie or a TV show episode,
//!   including metadata such as season and episode numbers for TV shows.
//!
//! - **`MakeMkv`**: A struct that manages the interaction with MakeMKV, including drive locking,
//!   output directory management, and the execution of ripping commands.
//!
//! - **`MAKE_MKV`**: A globally accessible, thread-safe instance of `MakeMkv` for managing
//!   ripping operations.
//!
//! ## Features
//!
//! - **Thread-Safe Drive Management**: Ensures that optical drives are locked during ripping
//!   operations to prevent concurrent access.
//!
//! - **Temporary Directory Handling**: Uses temporary directories for intermediate ripping
//!   output, ensuring clean-up after the process completes.
//!
//! - **Media Organization**: Automatically organizes ripped media into appropriate directories
//!   based on the type of rip (movie or TV show).
//!
//! - **Error Handling**: Provides detailed error types to handle various failure scenarios,
//!   such as missing MakeMKV installation, drive in use, or failed ripping operations.
//!
//! ## Usage
//!
//! To use this module, initialize the `MakeMkv` instance, configure the output directory,
//! and execute ripping operations using the `Rip` struct. The module is designed to work
//! asynchronously and integrates with the `tokio` runtime for concurrency.
//!
//! ## Example
//!
//! ```rust
//! use cord_ripper_v1::makemkv::makemkv_core::{Rip, RipType, MAKE_MKV};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize MakeMKV
//!     let mut makemkv = MAKE_MKV.lock().await;
//!     makemkv.init("/path/to/output/directory").await?;
//!
//!     // Create a Rip instance for a movie
//!     let rip = Rip {
//!         title: "My Movie".to_string(),
//!         drive_number: 1,
//!         rip_type: RipType::Movie,
//!         title_id: 1,
//!     };
//!
//!     // Execute the ripping process
//!     rip.execute().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Notes
//!
//! - Ensure that MakeMKV is installed and accessible on the system before using this module.
//! - The output directory must exist and be writable.
//! - This module is designed for asynchronous execution and requires a `tokio` runtime.
use core::panic;
use std::{collections::HashSet, path::PathBuf, sync::Arc, time::Instant};
// use tempdir::TempDir;
use tempfile::TempDir;
use tokio::sync::Mutex;

use crate::{debug, error, info, trace, warn};

use super::{
    errors::{MakeMkvError, Result},
    makemkv_helpers::{check_makemkv_output, makemkv_exists, Command as MakeMkvCommands},
};

lazy_static::lazy_static! {
    /// A globally accessible instance of `MakeMkv` for managing ripping operations.
    pub static ref MAKE_MKV: Arc<Mutex<MakeMkv>> = Arc::new(Mutex::new(MakeMkv::default()));
}

#[derive(Debug)]
pub struct Rip {
    pub title: String,
    pub drive_number: u8,
    pub rip_type: RipType,
    pub title_id: u16,
}

/// Represents a ripping operation, which can either be for a movie or a specific episode of a show.
///
/// The `Rip` struct provides functionality to execute the ripping process asynchronously
/// and retrieve metadata about the rip, such as the episode number if applicable.
///
/// # Methods
///
/// - `execute`: Executes the ripping process using the `MAKE_MKV` instance. This method
///   is asynchronous and returns a `Result` indicating the success or failure of the operation.
///
/// - `episode`: Returns the episode number if the rip is for a specific episode of a show.
///   If the rip is for a movie, this method returns `None`.
///
/// # Example
///
/// ```rust
/// let rip = Rip {
///     rip_type: RipType::Show { season: 1, episode: 5 },
///     // other fields...
/// };
///
/// // Execute the rip
/// rip.execute().await?;
///
/// // Get the episode number
/// if let Some(episode) = rip.episode() {
///     println!("Ripping episode {}", episode);
/// } else {
///     println!("Ripping a movie");
/// }
/// ```
///
/// This struct is designed to work with the `MAKE_MKV` instance, which handles the
/// underlying ripping logic.
impl Rip {
    pub async fn execute(&self) -> Result<()> {
        MAKE_MKV.lock().await.run_rip(self).await?;
        Ok(())
    }

    pub fn episode(&self) -> Option<u8> {
        match self.rip_type {
            RipType::Show { season: _, episode } => Some(episode),
            RipType::Movie => None,
        }
    }

    pub async fn cancel(&self) -> Result<()> {
        MAKE_MKV
            .lock()
            .await
            .unlock_drive(self.drive_number)
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum RipType {
    Movie,
    /// Represents a TV show with associated season and episode information.
    ///
    /// This enum is used to encapsulate metadata about a specific episode of a show,
    /// including the season and episode numbers. It is particularly useful for
    /// organizing and processing media content, such as in applications that handle
    /// TV series or episodic content.
    ///
    /// # Fields
    ///
    /// - `season`: The season number of the show (as an unsigned 8-bit integer).
    /// - `episode`: The episode number within the season (as an unsigned 8-bit integer).
    ///
    /// # Example
    ///
    /// ```rust
    /// let show = Show {
    ///     season: 1,
    ///     episode: 5,
    /// };
    /// println!("Season: {}, Episode: {}", show.season, show.episode);
    /// ```
    ///
    /// This will output:
    /// ```text
    /// Season: 1, Episode: 5
    /// ```
    Show {
        season: u8,
        episode: u8,
    },
}

pub struct MakeMkv {
    pub output_dir: PathBuf,
    pub drives: Arc<Mutex<HashSet<u8>>>,
}

impl Default for MakeMkv {
    fn default() -> Self {
        MakeMkv {
            output_dir: PathBuf::new(),
            drives: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

/// The `MakeMkv` struct provides functionality for interacting with the MakeMKV software
/// to rip media from optical drives. It manages the output directory for ripped files,
/// tracks locked drives to prevent concurrent access, and handles the ripping process.
///
/// # Fields
/// - `output_dir`: A `PathBuf` representing the directory where ripped files will be saved.
/// - `drives`: A thread-safe `HashSet` wrapped in an `Arc<Mutex<>>` to track locked drives.
///
/// # Methods
///
/// ## `new`
/// Creates a new instance of `MakeMkv`.
///
/// ### Parameters
/// - `output_dir`: A string slice representing the directory where ripped files will be saved.
///
/// ### Returns
/// A new `MakeMkv` instance.
///
/// ## `init`
/// Initializes the `MakeMkv` instance by verifying the existence of MakeMKV and the output directory.
///
/// ### Parameters
/// - `output_dir`: A string slice representing the directory where ripped files will be saved.
///
/// ### Returns
/// - `Ok(())` if initialization is successful.
/// - `Err(MakeMkvError)` if MakeMKV is not installed or the output directory does not exist.
///
/// ## `lock_drive`
/// Locks a specific drive to prevent concurrent access during the ripping process.
///
/// ### Parameters
/// - `drive_number`: A `u8` representing the drive number to lock.
///
/// ### Returns
/// - `Ok(())` if the drive is successfully locked.
/// - `Err(MakeMkvError)` if the drive is already in use.
///
/// ## `unlock_drive`
/// Unlocks a specific drive after the ripping process is complete.
///
/// ### Parameters
/// - `drive_number`: A `u8` representing the drive number to unlock.
///
/// ### Returns
/// - `Ok(())` if the drive is successfully unlocked.
///
/// ## `run_rip`
/// Executes the ripping process for a specific drive and title, saving the output to the appropriate directory.
///
/// ### Parameters
/// - `rip_details`: A reference to a `Rip` struct containing details about the drive, title, and rip type.
///
/// ### Returns
/// - `Ok(())` if the ripping process is successful.
/// - `Err(MakeMkvError)` if any error occurs during the ripping process.
///
/// ### Process
/// 1. Locks the specified drive.
/// 2. Creates a temporary output directory.
/// 3. Executes the MakeMKV command to rip the media.
/// 4. Validates the output and calculates ripping statistics.
/// 5. Moves the ripped file to the appropriate destination directory based on the rip type (movie or show).
/// 6. Unlocks the drive and cleans up temporary resources.
///
/// ### Errors
/// - Fails if MakeMKV command execution fails.
/// - Fails if no MKV files are found in the temporary output directory.
/// - Fails if the destination directory cannot be created or the ripped file cannot be moved.
///
/// ### Notes
/// - The method ensures thread safety by locking and unlocking drives during the ripping process.
/// - It calculates and logs ripping statistics, such as time taken and ripping speed.
impl MakeMkv {
    pub fn new(output_dir: &str) -> Self {
        let output_dir = PathBuf::from(output_dir);
        let drives = Arc::new(Mutex::new(HashSet::new()));
        MakeMkv { output_dir, drives }
    }

    pub async fn init(&mut self, output_dir: &str) -> Result<()> {
        let makemkv_exists = makemkv_exists().await;

        if !makemkv_exists {
            error!("MakeMKV is not installed");
            panic!("MakeMKV is not installed");
        }

        let output_dir = PathBuf::from(output_dir);

        if !output_dir.exists() {
            error!(
                "Output directory does not exist: {}",
                output_dir.to_string_lossy()
            );
            return Err(MakeMkvError::FileNotFoundError(
                output_dir.to_string_lossy().to_string(),
            ));
        }

        self.output_dir = output_dir;

        trace!(
            "Output directory set to: {}",
            self.output_dir.to_string_lossy()
        );
        info!("MakeMKV initialized successfully!");
        Ok(())
    }

    async fn lock_drive(&mut self, drive_number: u8) -> Result<()> {
        let mut drives = self.drives.lock().await;
        if drives.contains(&drive_number) {
            error!("Drive {} is already in use", drive_number);
            return Err(MakeMkvError::DriveInUseError(drive_number));
        }
        drives.insert(drive_number);
        debug!("Locked drive {}", drive_number);
        Ok(())
    }

    async fn unlock_drive(&mut self, drive_number: u8) -> Result<()> {
        let mut drives = self.drives.lock().await;
        drives.remove(&drive_number);
        debug!("Unlocked drive {}", drive_number);
        Ok(())
    }

    pub async fn run_rip(&mut self, rip_details: &Rip) -> Result<()> {
        info!(
            "Starting rip for drive {}: {}",
            rip_details.drive_number, rip_details.title
        );
        self.lock_drive(rip_details.drive_number).await?;

        let temp_output_dir = TempDir::with_prefix_in("makemkv_output", &self.output_dir)
            .map_err(|_| MakeMkvError::TempDirError)?;

        debug!(
            "Created temporary output directory: {}",
            temp_output_dir.path().display()
        );

        let dev_path = format!("dev:/dev/sr{}", rip_details.drive_number - 1);

        let title_id = rip_details.title_id - 1;

        let command = MakeMkvCommands::new(
            "makemkvcon",
            vec![
                "mkv".to_string(),
                dev_path,
                title_id.to_string(),
                "--minlength=600".to_string(),
                temp_output_dir.path().to_string_lossy().to_string(),
            ],
        );

        info!("Starting MakeMKV Command");
        debug!("Executing command: {} {:?}", command.command, command.args);
        let start_rip_time = Instant::now();

        let output = command.execute().await.map_err(|e| {
            error!("Failed to execute MakeMKV command: {}", e);
            MakeMkvError::CommandExecutionError(e.to_string())
        })?;

        self.unlock_drive(rip_details.drive_number).await?;

        trace!("MakeMKV output: {:?}", output);

        match check_makemkv_output(&output) {
            Ok(_) => (),
            Err(_) => {
                warn!("MakeMKV failed to rip {}!", rip_details.title);
                return Err(MakeMkvError::FailedToSaveDisc);
            }
        };

        let rip_size: f64 = fs_extra::dir::get_size(temp_output_dir.path())
            .map_err(|_| MakeMkvError::FailedToSaveDisc)? as f64
            / (1024.0 * 1024.0);
        let rip_time = start_rip_time.elapsed().as_secs_f64() / 60.00;
        let rate = rip_size / (rip_time * 60.00);

        info!(
            "Ripped {} in {:.2} minutes at {:.2} MB/s",
            rip_details.title, rip_time, rate
        );

        let ripped_files: Vec<PathBuf> = std::fs::read_dir(temp_output_dir.path())
            .map_err(|_| MakeMkvError::TempDirError)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "mkv"))
            .collect();

        if ripped_files.is_empty() {
            error!("No MKV files were found in the temporary output directory");
            return Err(MakeMkvError::FailedToSaveDisc);
        }

        let ripped_file = ripped_files.first().unwrap();
        debug!("Ripped file: {}", ripped_file.display());

        // This is garbage, please fix you lazy shit
        let (destination_dir, destination_path) = match rip_details.rip_type {
            RipType::Movie => (
                self.output_dir
                    .join(format!("movies/{}", rip_details.title)),
                self.output_dir
                    .join(format!(
                        "movies/{}/{}",
                        rip_details.title, rip_details.title
                    ))
                    .with_extension("mkv"),
            ),
            RipType::Show { season, episode } => (
                self.output_dir
                    .join(format!("shows/{}/Season {}", rip_details.title, season)),
                self.output_dir
                    .join(format!(
                        "shows/{}/Season {}/Episode {}",
                        rip_details.title, season, episode
                    ))
                    .with_extension("mkv"),
            ),
        };

        std::fs::create_dir_all(&destination_dir).map_err(|_| MakeMkvError::OutputDirError)?;

        debug!("Created output directory: {}", destination_dir.display());

        std::fs::rename(ripped_file, &destination_path)
            .map_err(|_| MakeMkvError::FailedToSaveDisc)?;
        debug!(
            "Moved ripped file from {} to {}",
            ripped_file.display(),
            destination_path.display()
        );

        temp_output_dir.close()?;
        debug!("Closed temporary output directory");

        info!("Successfully ripped {}!", rip_details.title);

        Ok(())
    }
}
