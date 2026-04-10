use std::path::PathBuf;

use aetherium_engine::{
    Engine, EngineLiveRequest, EngineRequest, EngineResponse,
    types::{EngineEvent, SyncProgress},
};
use anyhow::anyhow;
use clap::Subcommand;

use crate::{
    commands::{
        AddFile, DeleteFile, ListFile, Runnable, SearchCmd, SyncCodex, cluster::ClusterCmd,
    },
    views::tabular_view::{Column, TabularView},
};

#[derive(Subcommand)]
pub enum CodexCmd {
    Add(AddFile),
    Delete(DeleteFile),
    Sync(SyncCodex),
    ListFiles(ListFile),
    #[command(subcommand, name = "cluster")]
    Cluster(ClusterCmd),
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
            ListFiles(s) => {
                if s.no_cluster {
                    return listfile_function(&engine, &path).await;
                }
                return listfile_with_cluster(&engine, &path).await;
            }
            Search(s) => s.run().await,
            Delete(s) => deletefile_function(&engine, &path, s.file.clone()).await,
            Cluster(c) => c.run().await,
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
    let mut table = TabularView::new(vec![
        Column::new("id"),
        Column::new("name"),
        Column::new("extention"),
        Column::new("modified at"),
    ]);

    match response {
        EngineResponse::FileList { files } => {
            for file in files {
                table.add_row(vec![
                    Some(file.id),
                    Some(file.name),
                    Some(file.extension),
                    file.modified_at,
                ])?;
            }

            table.print();

            Ok(())
        }

        EngineResponse::Error { message } => return Err(anyhow!(message)),

        _ => return Err(anyhow!("unexpected response")),
    }
}

async fn listfile_with_cluster(engine: &Engine, path: &PathBuf) -> anyhow::Result<()> {
    let response = engine
        .handle(EngineRequest::ListFileWithClusters {
            codex_path: path.to_path_buf(),
        })
        .await;

    let mut table = TabularView::new(vec![
        Column::new("id"),
        Column::new("name"),
        Column::new("extention"),
        Column::new("created at"),
        Column::new("Cluster Name"),
        Column::new("Match"),
    ]);

    match response {
        EngineResponse::FileListWithClusters { files } => {
            for file in files {
                table.add_row(vec![
                    Some(file.id),
                    Some(file.name),
                    Some(file.extension),
                    file.created_at,
                    file.cluster_name,
                    file.top_cluster_pct.map(|p| format!("{:.0}%", p)),
                ])?;
            }

            table.print();

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
        .handle_live(
            EngineLiveRequest::AddFile {
                codex_path: path.clone(),
                file_path: PathBuf::from(addfile.file.clone()),
                file_name: addfile.name.clone(),
            },
            &|event| match event {
                EngineEvent::OperationStarted => {
                    println!(
                        " -> Adding File {} ({})",
                        addfile.file,
                        addfile
                            .name
                            .clone()
                            .unwrap_or("#Not Specified#".to_string())
                    )
                }
                EngineEvent::MLUnavailable => {
                    println!(
                        "ML Server is unavailable currently... run 'aetherium ml-server start'"
                    );
                    println!("Embeddings wont be performed");
                }

                _ => {
                    eprintln!("Unexpected event Recorded")
                }
            },
        )
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
                EngineEvent::Sync(SyncProgress::DimsMISMATCH { previous, proposed }) => {
                    println!("@# DIMS mismatch found; current: {}, given: {}; reseting table", previous, proposed)
                }
                EngineEvent::Sync(SyncProgress::DimsChanged { previous, now }) => {
                    println!("@# Embedding Table Has Been Reset")
                }
                EngineEvent::MLUnavailable => {

                    println!(
                        "ML Server is unavailable currently... run 'aetherium ml-server start'"
                    );

                    println!("Embedding wont be performed... run 'aetherium codex sync after starting ml-server'")
                }

                EngineEvent::Sync(SyncProgress::Embedding { file_id }) => {
                    println!(" [...] embedding file = {file_id}")
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

        EngineResponse::PartialSynced => {
            println!("Partially Synced, Embeddings Left");
            Ok(())
        }

        EngineResponse::Error { message } => return Err(anyhow!(message)),

        _ => return Err(anyhow!("unexpected response")),
    }
}
