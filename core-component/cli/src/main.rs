// TODO: Create a simple CLI App that Interacts with the core-component/engine
// This is the starting tool for the user to interact with the main aetherium component

// TODO: Make It functional, add boiler plate for interacting with engine library

// TODO: AFTER ENGINE IMPLEMENTATION: Interaction with the engine, and print when the work is don

use clap::Parser;
use cli::{CLI, Commands, commands::Runnable};
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

#[cfg(test)]
mod testing {

    use cli::views::content_viewer::{ContentTable, MetaDataCell, Render};
    use terminal_size::{Height, Width, terminal_size};

    #[test]
    pub fn content_render_check() {
        // will always fail, just to check how the content is rendering
        let mut width: usize;
        let mut height: usize;
        if let Some((Width(w), Height(h))) = terminal_size() {
            width = w as usize;
            height = h as usize;
        } else {
            println!("Got error during width and height");
            width = 800 as usize;
            height = 600 as usize;
        }

        println!("terminal width: {}, terminal height {}", width, height);
        let meta_data_cell = MetaDataCell::new(
            "Some Cool Title",
            "Some Cool Subtitle",
            vec![String::from("Tag"), String::from("New Tag")],
            None,
            None,
        );

        let meta_data_cell2 = MetaDataCell::new(
            "Some Cool Title",
            "Some Cool Subtitle",
            vec![String::from("Tag"), String::from("New Tag")],
            Some("New Information Gained I Want to show".to_string()),
            None,
        );
        let content_table = ContentTable::build_relative_window(
            vec![
                (
                    meta_data_cell,
                    vec![
                        String::from("Some Content"),
                        String::from("Some Content"),
                        String::from("Some Content"),
                        String::from("Some Content"),
                    ],
                ),
                (
                    meta_data_cell2,
                    vec![
                        String::from("Some Content"),
                        String::from("Some Content"),
                        String::from("Some Content"),
                        String::from("Some Content"),
                        String::from("Some Content"),
                        String::from("Some Content"),
                    ],
                ),
            ],
            None,
            (None, None),
            width as u16,
            height as u16,
        );

        println!("{}", content_table.render());
        assert_eq!(1, 0)
    }
}
