use crate::{SearchMode, commands::Runnable};
use clap::Args;

#[derive(Args)]
pub struct AskCmd {
    pub query: String,

    #[arg(long)]
    pub ignore_case: bool,

    #[arg(short = 'k', long, default_value_t = 10)]
    pub top_k: usize,

    #[arg(short, long, value_enum, default_value = "hybrid")]
    pub mode: SearchMode,
}

impl Runnable for AskCmd {}
