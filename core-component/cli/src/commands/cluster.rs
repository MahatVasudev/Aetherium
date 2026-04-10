use std::{any, path::Path};

use crate::{
    commands::Runnable,
    views::{
        config::{
            ContentCellConfig, ContentTableCellConfig, ContentTableConfig, MetaDataCellConfig,
        },
        content_viewer::{ContentTable, MetaDataCell, Render},
        tabular_view::{Column, TabularView},
        types::AllignmentType,
    },
};
use aetherium_engine::{
    Engine, EngineLiveRequest, EngineRequest, EngineResponse,
    types::{EngineEvent, SyncProgress},
};
use anyhow;
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum ClusterCmd {
    List,
    Stats,
    Reset,
    ListFiles(ListClusteredFiles),
}

#[derive(Args)]
pub struct ListClusteredFiles {
    #[arg(long, short, value_names = ["KEY"])]
    pub clustername: Option<String>,

    #[arg(long, short, value_names = ["KEY"])]
    pub clusterid: Option<i32>,
}

#[async_trait::async_trait]
impl Runnable for ClusterCmd {
    async fn run(&self) -> anyhow::Result<()> {
        let path = std::env::current_dir()?;
        let engine = Engine::new(None);

        match self {
            ClusterCmd::List => handle_list_cluster(path, &engine),
            ClusterCmd::Reset => handle_reset_cluster(path, &engine).await,
            ClusterCmd::ListFiles(lcf) => handle_lcf_cluster(path, &engine, &lcf),
            ClusterCmd::Stats => handle_cluster_stats(path, &engine).await,
        }
    }
}

async fn handle_cluster_stats<P: AsRef<Path>>(path: P, engine: &Engine) -> anyhow::Result<()> {
    let response = engine
        .handle(EngineRequest::ClusterStats {
            codex_path: path.as_ref().to_path_buf(),
        })
        .await;

    match response {
        EngineResponse::ClusterStats { model_info, stats } => {
            let mut table = TabularView::new(vec![
                Column::new("cluster method name"),
                Column::new("dimensional reduction name"),
            ]);

            table.add_row(vec![
                Some(model_info.name),
                Some(model_info.dimension_reduction_model),
            ])?;

            let mut ctv = ContentTable::build(
                stats
                    .iter()
                    .rev()
                    .map(|f| {
                        (
                            MetaDataCell::new(
                                &f.id.to_string(),
                                &f.name,
                                vec![],
                                Some(f.created_at.to_owned()),
                                Some(MetaDataCellConfig {
                                    max_letters: std::cmp::max(
                                        f.name.chars().count(),
                                        f.created_at.chars().count(),
                                    ),
                                    ..Default::default()
                                }),
                            ),
                            vec![
                                format!("File Count\n{}", f.file_count),
                                format!("Chunks Count\n{}", f.chunk_count),
                                f.top_files.join("\n"),
                            ],
                        )
                    })
                    .collect::<Vec<(_, _)>>(),
                Some(ContentTableConfig {
                    row_separator: ">".to_string(),
                    ..Default::default()
                }),
                (
                    Some(ContentTableCellConfig {
                        vertical_border_style: "$".to_string(),
                        padding: 2,
                        ..Default::default()
                    }),
                    Some(ContentCellConfig {
                        content_allignment: AllignmentType::CENTER,
                        ..Default::default()
                    }),
                ),
            );

            println!("Each Cluster Info");
            println!("{}", ctv.render());
            println!("Cluster Metadata Info");
            println!("{}", table.render()?);
            println!(
                "dimensions reduced {} -> {}",
                model_info.dims, model_info.reduced_to
            )
        }

        EngineResponse::Error { message } => {
            eprintln!("Error Received, Clustering Failed:\n{message}")
        }

        _ => {
            eprintln!("Unexpected Response Received")
        }
    }

    Ok(())
}

fn handle_list_cluster<P: AsRef<Path>>(path: P, engine: &Engine) -> anyhow::Result<()> {
    Ok(())
}

async fn handle_reset_cluster<P: AsRef<Path>>(path: P, engine: &Engine) -> anyhow::Result<()> {
    let response = engine
        .handle_live(
            EngineLiveRequest::Cluster {
                codex_path: path.as_ref().to_path_buf(),
            },
            &|event| match event {
                EngineEvent::OperationStarted => {
                    println!("Starting Clustering, Preparing Resources....")
                }
                EngineEvent::Clustering => {
                    println!("Finally Clustering Files... Please Wait...")
                }
                EngineEvent::OperationFinished => {
                    println!("Clustering Operation Successfull...")
                }

                _ => {
                    eprintln!("Unexpected Response Received")
                }
            },
        )
        .await;

    match response {
        EngineResponse::Clustered {
            clusters_found,
            unique_clusters,
        } => {
            let mut table =
                TabularView::new(vec![Column::new("cluster id"), Column::new("cluster name")]);

            for (cluster_name, cluster_id) in &unique_clusters {
                table.add_row(vec![
                    Some(cluster_id.to_string()),
                    Some(cluster_name.to_string()),
                ])?;
            }

            println!("Unique clusters found {}", clusters_found);

            table.print();
        }

        EngineResponse::Error { message } => {
            eprintln!("Error Received, Clustering Failed:\n{message}")
        }

        _ => {
            eprintln!("Unexpected Response")
        }
    }

    Ok(())
}

fn handle_lcf_cluster<P: AsRef<Path>>(
    path: P,
    engine: &Engine,
    lcf: &ListClusteredFiles,
) -> anyhow::Result<()> {
    Ok(())
}
