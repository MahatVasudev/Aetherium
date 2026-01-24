// TODO: Create a simple CLI App that Interacts with the core-component/engine
// This is the starting tool for the user to interact with the main aetherium component

// TODO: Make It functional, add boiler plate for interacting with engine library

// TODO: AFTER ENGINE IMPLEMENTATION: Interaction with the engine, and print when the work is don

use clap::Parser;
use cli::{CLI, Commands, commands::CodexCmd};
// TODO: MAKE IT PRETTY
fn main() {
    let cli = CLI::parse();

    match cli.command {
        Commands::Create(cmd) => {
            println!("Create: {}", cmd.codex)
        }
        Commands::Ask(val) => {
            println!(
                "Ask: {}, options ignore_case: {}, mode: {:?}, asked results: {}",
                val.query, val.ignore_case, val.mode, val.top_k
            )
        }
        Commands::Search(cmd) => {
            println!("Search Query: {}, files: {:?}", cmd.query, cmd.files)
        }
        Commands::Codex(cmd) => match cmd {
            CodexCmd::Add(val) => println!("Adding File: {}", val.file),
            CodexCmd::Delete(val) => println!(
                "Deleting File: {}, with conformation: {}",
                val.file, val.conformation
            ),
        },
    }
}
