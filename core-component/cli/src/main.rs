// TODO: Create a simple CLI App that Interacts with the core-component/engine
// This is the starting tool for the user to interact with the main aetherium component

// TODO: Make It functional, add boiler plate for interacting with engine library

// TODO: AFTER ENGINE IMPLEMENTATION: Interaction with the engine, and print when the work is don

use aetherium_engine::{Engine, EngineRequest};
use clap::Parser;
use cli::{
    CLI, Commands,
    commands::{CodexCmd, Runnable},
};
// TODO: MAKE IT PRETTY

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let cli = CLI::parse();
    let result = match cli.command {
        Commands::Create(cmd) => cmd.run().await,
        Commands::Codex(cmd) => cmd.run().await,
        Commands::Config(cmd) => cmd.run().await,
        Commands::MLServer(cmd) => cmd.run().await,
    };

    if let Err(e) = result {
        eprintln!("Error received: {}", e);
        std::process::exit(1);
    }
}
