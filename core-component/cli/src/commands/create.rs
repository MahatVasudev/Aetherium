use clap::Args;

use crate::commands::Runnable;

#[derive(Args)]
pub struct CreateCmd {
    pub codex: String,
}

impl Runnable for CreateCmd {}
