use serenity::all::CreateCommand;

use crate::debug;

pub fn register() -> CreateCommand {
    debug!("Regisered eject_disc command");
    CreateCommand::new("eject_disc").description("Eject the disc from the drive")
}

pub fn run() {
    debug!("Running eject_disc command");
}
