use std::fs;

use aetherium_engine::{Engine, EngineRequest, EngineResponse};
use anyhow::anyhow;
use async_trait::async_trait;
use clap::Args;

use crate::commands::{AddFile, DeleteFile, Runnable};

#[derive(Args)]
pub struct ConfigCmd {
    #[arg(long,short, value_names = ["KEY"])]
    pub get: Option<String>,
    #[arg(long,short, value_names = ["KEY", "VALUE"], num_args=2)]
    pub set: Option<Vec<String>>,
}

#[async_trait::async_trait]
impl Runnable for ConfigCmd {
    async fn run(&self) -> anyhow::Result<()> {
        let path = std::env::current_dir()?;

        let engine = Engine::new(None);
        match (&self.get, &self.set) {
            (Some(key), None) => {
                let get_value = engine
                    .handle(EngineRequest::GetConfig {
                        codex_path: path,
                        key: key.clone(),
                    })
                    .await;
                match get_value {
                    EngineResponse::GotConfig { value } => match value {
                        None => println!(),
                        Some(v) => println!("{}", v),
                    },
                    EngineResponse::Error { message } => return Err(anyhow::anyhow!(message)),
                    _ => return Err(anyhow::anyhow!("found different response")),
                };
            }
            (None, Some(pair)) => {
                let key = &pair[0];
                let value = &pair[1];

                let result = engine
                    .handle(EngineRequest::SetConfig {
                        codex_path: path,
                        key: key.into(),
                        val: value.into(),
                    })
                    .await;

                match result {
                    EngineResponse::SettedConfig { key, val } => {
                        println!("successfully changed {key} => {val}");
                    }
                    EngineResponse::Error { message } => {
                        return Err(anyhow::anyhow!("could not set the key: {}", message));
                    }

                    _ => return Err(anyhow::anyhow!("found different response")),
                };
            }
            (Some(_), Some(_)) => return Err(anyhow::anyhow!("cannot do both at the same time!")),
            (None, None) => return Err(anyhow::anyhow!("do atleast one!")),
        };

        Ok(())
    }
}
