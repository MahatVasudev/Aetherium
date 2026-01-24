use crate::commands::Runnable;
use clap::Args;

#[derive(Args)]
pub struct SearchCmd {
    pub query: String,

    #[arg(short = 'f', long)]
    pub files: Option<String>,
}

impl Runnable for SearchCmd {}
