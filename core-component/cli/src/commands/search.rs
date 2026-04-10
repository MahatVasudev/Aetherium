use std::env;

use crate::commands::Runnable;
use crate::views::config::MetaDataCellConfig;
use crate::views::content_viewer::ContentTable;
use crate::views::content_viewer::MetaDataCell;
use crate::views::content_viewer::Render;

use aetherium_engine::Engine;
use aetherium_engine::EngineRequest;
use aetherium_engine::EngineResponse;
use anyhow::anyhow;
use clap::Args;
use clap::ValueEnum;
use terminal_size::Height;
use terminal_size::Width;
use terminal_size::terminal_size;

#[derive(Args)]
pub struct SearchCmd {
    pub query: String,

    #[arg(short = 'f', long)]
    pub files: Option<String>,

    #[arg(long, default_value_t = 10)]
    pub top_k: usize,

    #[arg(short = 'm', long, default_value = "lexical")]
    pub mode: SearchMode,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum SearchMode {
    #[value(alias = "l", alias = "literal")]
    Lexical,
    #[value(alias = "s", alias = "dynamic", alias = "question")]
    Semantic,
    #[value(alias = "m", alias = "all")]
    Mix,
}

impl SearchMode {
    fn as_str(&self) -> String {
        match self {
            SearchMode::Lexical => "lexical".into(),
            SearchMode::Semantic => "semantic".into(),
            SearchMode::Mix => "mix".into(),
        }
    }
}

#[async_trait::async_trait]
impl Runnable for SearchCmd {
    async fn run(&self) -> anyhow::Result<()> {
        let current_path = env::current_dir()?;
        let engine = Engine::new(None);

        match engine
            .handle(EngineRequest::SearchFiles {
                codex_path: current_path.to_string_lossy().to_string(),
                query: self.query.clone(),
                query_type: self.mode.as_str(),
                top_k: self.top_k,
            })
            .await
        {
            EngineResponse::SearchResults { results } => {
                let contents = results
                    .iter()
                    .map(|f| {
                        (
                            MetaDataCell::new(
                                &f.0.file_name,
                                &f.0.file_id,
                                vec![
                                    f.0.cluster
                                        .to_owned()
                                        .unwrap_or("Not Clustered Yet".to_string()),
                                ],
                                Some(format!(
                                    "Chunk id {} Distance id {}",
                                    f.0.chunk_id, f.0.distance
                                )),
                                Some(MetaDataCellConfig {
                                    max_letters: f.0.file_id.len(),
                                    ..Default::default()
                                }),
                            ),
                            vec![f.1.to_owned()],
                        )
                    })
                    .collect::<Vec<(MetaDataCell, _)>>();
                let content_table = ContentTable::build(contents, None, (None, None));

                println!("{}", content_table.render());
                Ok(())
            }

            EngineResponse::Error { message } => Err(anyhow!(message)),

            _ => Err(anyhow!("unexpected message")),
        }
    }
}
