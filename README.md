# Cord Ripper

Cord Ripper is a tool designed to automate the process of ripping discs and managing media files. It integrates with Discord to provide a user-friendly interface for managing disc ripping operations.

## Version

0.0.9 Beta

## Features

- Rip movies and TV shows remotely within Discord.
- View available titles on a disc.
- Manage ripping operations via Discord commands.
- Supports MakeMKV for disc operations.

## Prerequisites

- [Rust](https://www.rust-lang.org/) installed on your system.
- [MakeMKV](https://forum.makemkv.com/forum/viewtopic.php?f=3&t=224) installed and accessible via the command line.
- A Linux distro compatible with MakeMKV.
- A Discord bot token and a valid guild ID.

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/qmusillo/cord-ripper.git
   cd cord-ripper
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Set the required environment variables:
   ```bash
   export DISCORD_TOKEN=your_discord_bot_token
   export GUILD_ID=your_guild_id
   ```

## Usage

1. Run the program:
   ```bash
   cargo run --release -- --output-dir /path/to/output
   ```
   or
   ```bash
   /path/to/repo/target/release/cord-ripper --output-dir /path/to/output
   ```

2. Use the Discord bot to interact with the program:
   - `/rip` to start a ripping operation.
   - `/get_titles` to view available titles on a disc.
   - `/view_drives` to list available drives.

## Known Issues

Below are some known issues and limitations of Cord Ripper v1:

- Errors may occur if MakeMKV is not properly installed or configured.
- The Discord bot may fail to respond if the token or guild ID is incorrect.
- Limited support for non-standard disc formats.
- Some drives may not be recognized depending on the system configuration.
- Titles may rip out of order based on the disc's file layout.

If you encounter any issues not listed here, please report them via the [GitHub Issues](https://github.com/qmusillo/cord-ripper/issues) page.

## Future Updates

The following features and improvements are planned for future releases of Cord Ripper in order of perceived priority:

- **Improved error handling**: Enhance error messages and recovery mechanisms for better user experience.
- **Enhanced logging**: Introduce log rotation and archiving to manage log files efficiently.
- **Ejecting Discs**: Enable disc tray manipulation and cycling via the Discord GUI.
- **Automated updates**: Implement a system for checking and applying updates automatically.

If you have suggestions for additional features, feel free to submit them via the [GitHub Issues](https://github.com/qmusillo/cord-ripper/issues) page.

## Limitations

- Requires a stable internet connection for Discord bot functionality.
- Only supports systems with MakeMKV installed and configured.
- Limited to systems running Linux.
- No built-in functionality for transcoding or compressing ripped files.
- Requires manual configuration of environment variables.
- May not work with older or unsupported disc drives.
- Performance may vary depending on system hardware and disc quality.
- Logs are not automatically rotated or archived.
- Limited error handling for unexpected system configurations.
- Does not include automated updates or patching mechanisms.


## Disclaimer

This tool is intended for personal use only. Ensure you comply with all applicable laws and regulations when using this software.
