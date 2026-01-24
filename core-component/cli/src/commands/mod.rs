mod ask;
mod codex;
mod create;
mod search;

pub use crate::commands::{ask::AskCmd, codex::CodexCmd, create::CreateCmd, search::SearchCmd};
use anyhow;
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Create(CreateCmd),
    Ask(AskCmd),
    Search(SearchCmd),
    #[command(subcommand)]
    Codex(CodexCmd),
}

pub trait Runnable {
    fn run(&self) -> anyhow::Result<()> {
        unimplemented!()
    }
}

#[derive(Args)]
pub struct AddFile {
    pub file: String,
}

#[derive(Args)]
pub struct DeleteFile {
    pub file: String,

    #[arg(short = 'y')]
    pub conformation: bool,
}
