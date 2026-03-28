use std::path::PathBuf;

use aetherium_engine::{
    Engine, EngineLiveRequest, EngineRequest, EngineResponse,
    types::{EngineEvent, SyncProgress},
};
use anyhow::anyhow;
use async_trait::async_trait;
use clap::Subcommand;

use crate::commands::{AddFile, DeleteFile, Runnable, SearchCmd, SyncCodex};

#[derive(Subcommand)]
pub enum CodexCmd {
    Add(AddFile),
    Delete(DeleteFile),
    Sync(SyncCodex),
    ListFiles,

    #[command(name = "ask")]
    Search(SearchCmd),
}

#[async_trait::async_trait]
impl Runnable for CodexCmd {
    async fn run(&self) -> anyhow::Result<()> {
        use CodexCmd::*;
        let path = std::env::current_dir()?;
        let engine = Engine::new(None);
        match self {
            Add(addfile) => addfile_function(addfile, &engine, &path).await,
            Sync(s) => sync_function(&s, &engine, &path).await,
            ListFiles => listfile_function(&engine, &path).await,
            Search(s) => s.run().await,
            Delete(s) => deletefile_function(&engine, &path, s.file.clone()).await,
            _ => return Err(anyhow!("Not Implemented yet")),
        }
    }
}

async fn deletefile_function(
    engine: &Engine,
    path: &PathBuf,
    file_id: String,
) -> anyhow::Result<()> {
    let response = engine
        .handle(EngineRequest::DeleteFile {
            codex_path: path.to_path_buf(),
            file_id: file_id.clone(),
        })
        .await;

    match response {
        EngineResponse::FileDeleted => {
            println!("file deleted {}", file_id);
            Ok(())
        }

        EngineResponse::Error { message } => Err(anyhow!(message)),

        _ => Err(anyhow!("unexpected response")),
    }
}

async fn listfile_function(engine: &Engine, path: &PathBuf) -> anyhow::Result<()> {
    let response = engine
        .handle(EngineRequest::ListFiles {
            codex_path: path.to_path_buf(),
        })
        .await;

    match response {
        EngineResponse::FileList { files } => {
            println!("id\tname\textension\tmodified at");
            for file in files {
                let modified_at = match file.modified_at {
                    Some(date) => date,
                    None => "--".to_string(),
                };
                println!(
                    "{}\t{}\t{}\t{:?}",
                    file.id, file.name, file.extension, modified_at
                );
            }

            Ok(())
        }

        EngineResponse::Error { message } => return Err(anyhow!(message)),

        _ => return Err(anyhow!("unexpected response")),
    }
}

async fn addfile_function(
    addfile: &AddFile,
    engine: &Engine,
    path: &PathBuf,
) -> anyhow::Result<()> {
    let response = engine
        .handle(EngineRequest::AddFile {
            codex_path: path.clone(),
            file_path: PathBuf::from(addfile.file.clone()),
            file_name: addfile.name.clone(),
        })
        .await;

    match response {
        EngineResponse::FileAdded { file_id, hash } => {
            println!("Added File {} \n hash: {}", file_id, hash);
            Ok(())
        }

        EngineResponse::Error { message } => return Err(anyhow!(message)),

        _ => return Err(anyhow!("unexpected response")),
    }
}

async fn sync_function(
    sync_type: &SyncCodex,
    engine: &Engine,
    path: &PathBuf,
) -> anyhow::Result<()> {
    let mut response: EngineResponse;
    if sync_type.silent {
        response = engine
            .handle(EngineRequest::Sync {
                codex_path: path.clone(),
            })
            .await;
    } else {
        response = engine.handle_live(
            EngineLiveRequest::Sync {
                codex_path: path.clone(),
            },
            &|event| match event {
                EngineEvent::Sync(SyncProgress::FileAdded { id, name }) => {
                    println!(" + Added File = {id} | {name}")
                }
                EngineEvent::Sync(SyncProgress::FileRemoved { id }) => {
                    println!(" - Removed File = {id}")
                }
                EngineEvent::Sync(SyncProgress::FileUpdated { id }) => {
                    println!(" * File Updated = {id}")
                }
                EngineEvent::Sync(SyncProgress::Done {
                    added,
                    removed,
                    updated,
                }) => {
                    println!(
                        "Files Synced: \nAdded: {added} \nRemoved: {removed} \nUpdated: {updated}"
                    )
                }
                _ => (),
            },
        ).await;
    }

    match response {
        EngineResponse::Synced => {
            println!("Files Synced");
            Ok(())
        }

        EngineResponse::Error { message } => return Err(anyhow!(message)),

        _ => return Err(anyhow!("unexpected response")),
    }
}
