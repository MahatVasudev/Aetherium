use clap::Subcommand;

use crate::commands::{AddFile, DeleteFile, Runnable};

#[derive(Subcommand)]
pub enum CodexCmd {
    Add(AddFile),
    Delete(DeleteFile),
}

impl Runnable for CodexCmd {}
