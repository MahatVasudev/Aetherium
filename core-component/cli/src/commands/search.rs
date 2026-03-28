use std::env;

use crate::commands::Runnable;

use aetherium_engine::Engine;
use aetherium_engine::EngineRequest;
use aetherium_engine::EngineResponse;
use anyhow::anyhow;
use clap::Args;
use clap::ValueEnum;

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
            EngineResponse::SearchResults => {
                println!("Successfull");
                Ok(())
            }

            EngineResponse::Error { message } => Err(anyhow!(message)),

            _ => Err(anyhow!("unexpected message")),
        }
    }
}
