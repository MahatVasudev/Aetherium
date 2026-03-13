use async_trait::async_trait;
use std::path::PathBuf;

use aetherium_engine::{Engine, EngineRequest, EngineResponse};
use clap::Args;

use crate::commands::Runnable;

#[derive(Args)]
pub struct CreateCmd {
    #[arg(long, short)]
    pub path: Option<String>,
    #[arg(long, short)]
    pub name: Option<String>,

    #[arg(long, default_value = "v1.0.0")]
    pub codex_version: String,

    #[arg(long, default_value = "v1")]
    pub storage_version: String,

    #[arg(long, default_value = "v1")]
    pub sqlitestore_version: String,
}

#[async_trait::async_trait]
impl Runnable for CreateCmd {
    async fn run(&self) -> anyhow::Result<()> {
        let path = match &self.path {
            None => std::env::current_dir()?,
            Some(p) => PathBuf::from(p),
        };

        let full_path = match &self.name {
            Some(n) if n != "." => path.join(n),
            _ => path,
        };

        let engine = Engine::new(None);

        let response = engine
            .handle(EngineRequest::CreateCodex {
                path: full_path,
                codex_version: self.codex_version.clone(),
                storage_version: self.storage_version.clone(),
                sqlite_version: self.sqlitestore_version.clone(),
            })
            .await;

        match response {
            EngineResponse::Error { message } => Err(anyhow::anyhow!(message)),
            EngineResponse::CodexCreated { id, name } => {
                println!("Codex created id: {}, name: {}", id, name);
                Ok(())
            }
            _ => Err(anyhow::anyhow!("unexpected response")),
        }
    }
}
