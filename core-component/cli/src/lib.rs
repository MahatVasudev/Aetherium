pub mod commands;

use clap::{Parser, ValueEnum};
pub use commands::Commands;

#[derive(Parser)]
#[command(name = "Aetherium", version = "V1", about = "Knowledge Engine")]
pub struct CLI {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum SearchMode {
    Lexical,
    Semantic,
    Hybrid,
}
