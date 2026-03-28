use aetherium_engine::{Engine, EngineRequest, EngineResponse};
use anyhow::anyhow;
use clap::{Args, Subcommand};

use crate::commands::Runnable;

#[derive(Subcommand)]
pub enum MLCmd {
    #[command(name = "health")]
    CheckHealth,

    Config(MLConfig),
}

#[derive(Args)]
pub struct MLConfig {
    #[arg(long, short, value_names = ["KEY"])]
    pub get: Option<String>,

    #[arg(long, short, value_names = ["KEY", "VALUE"], num_args = 2)]
    pub set: Option<Vec<String>>,
}

#[async_trait::async_trait]
impl Runnable for MLCmd {
    async fn run(&self) -> anyhow::Result<()> {
        let path = std::env::current_dir()?;
        let engine = Engine::new(None);
        match self {
            MLCmd::CheckHealth => ml_checkhealth(&engine).await,
            MLCmd::Config(value) => todo!(),
        }
    }
}

async fn ml_checkhealth(engine: &Engine) -> anyhow::Result<()> {
    match engine.handle(EngineRequest::MLHealth).await {
        EngineResponse::MLHealth {
            status,
            version,
            model,
            dims,
        } => {
            println!("ML Server Working...");
            println!(
                "status: {}; version: {}; model: {}, dims: {}",
                status, version, model, dims
            );
            Ok(())
        }

        EngineResponse::Error { message } => return Err(anyhow!(message)),

        _ => return Err(anyhow!("unexpected response")),
    }
}
