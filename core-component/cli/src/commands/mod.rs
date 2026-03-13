mod ask;
mod codex;
pub mod config;
mod create;
mod search;

use crate::commands::config::ConfigCmd;
pub use crate::commands::{ask::AskCmd, codex::CodexCmd, create::CreateCmd, search::SearchCmd};
use anyhow;
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "create-codex")]
    Create(CreateCmd),
    #[command(subcommand)]
    Codex(CodexCmd),
    Config(ConfigCmd),
}

#[async_trait::async_trait]
pub trait Runnable {
    async fn run(&self) -> anyhow::Result<()> {
        unimplemented!()
    }
}

#[derive(Args)]
pub struct AddFile {
    pub file: String,

    #[arg(long, short)]
    pub name: Option<String>,
}

#[derive(Args)]
pub struct SyncCodex {
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub silent: bool,
}

#[derive(Args)]
pub struct DeleteFile {
    pub file: String,

    #[arg(short = 'y')]
    pub conformation: bool,
}
