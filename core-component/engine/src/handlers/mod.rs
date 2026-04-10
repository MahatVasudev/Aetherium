pub mod grpc_ops;
use crate::ml_client::MLClient;
use aetherium_core::{
    CONFIG_DIR,
    codex::{Codex, versions::CodexVersion},
    ml_server::config::{DimensionsReductionModel, MLConfig},
    storage::{
        error::{SqliteError, StorageError},
        sqlite_version::SqliteStoreVersion,
        storage_types::SyncEvent,
        versions::StorageVersion,
    },
    tfidf::{
        TFIDFCorpus,
        chunkreader::ChunkReader,
        embeddings::{Chunk, ChunkEmbedding},
        sentence_chunker::SentenceChunkerBatcher,
    },
};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    sync::Mutex,
};
use tonic::IntoRequest;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    EngineResponse, engine_err,
    error::EngineError,
    types::{
        ClusterChunkInput, DocTextChunk, EngineEvent, FileDetail, FileDetailWithCluster,
        SearchMatchedDetails, SearchType, SyncProgress, VectorInput,
    },
    utils::{self, embed_file_helper},
};

pub fn handler_create_codex<P: AsRef<Path>>(
    path: P,
    codex_version: &str,
    storage_version: &str,
    sqlite_version: &str,
) -> EngineResponse {
    let codex_version = match CodexVersion::parse(&codex_version) {
        None => {
            warn!(
                "Received a non valid Codex Version, Defaulting to {}",
                CodexVersion::latest().as_str()
            );
            CodexVersion::latest()
        }
        Some(v) => v,
    };

    // FIX: Add implementation of Kmeans and Dbscan
    let storage_version = match StorageVersion::parse(&storage_version) {
        None => {
            warn!(
                "Received a non valid Storage Version, Defaulting to {}",
                StorageVersion::latest().as_str()
            );
            StorageVersion::latest()
        }
        Some(v) => v,
    };

    let sqlite_version = match SqliteStoreVersion::parse(&sqlite_version) {
        None => {
            warn!(
                "Received a non valid SqliteStore Version, Defaulting to {}",
                SqliteStoreVersion::latest().as_str()
            );
            SqliteStoreVersion::latest()
        }
        Some(v) => v,
    };
    let full_path = path.as_ref();
    let codex = match Codex::build(&full_path, codex_version, storage_version, sqlite_version) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while building codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    info!(
        "Codex Successfully Created: Details id: {}, name: {}, path: {}, version: {}, storage_version: {}, sqlitestore_version: {}",
        codex.id,
        codex.name,
        codex.storage.root_folder().to_str().unwrap(),
        codex.version().as_str(),
        codex.storage.version().as_str(),
        codex.storage.sqlite().unwrap().version().as_str()
    );

    return EngineResponse::CodexCreated {
        id: codex.id,
        name: codex.name,
    };
}

pub fn handler_open_codex<P: AsRef<Path>>(path: P) -> EngineResponse {
    let codex = match Codex::open(path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    info!(
        "Codex Successfully Opened: Details id: {}, name: {}, path: {}, version: {}, storage_version: {}, sqlitestore_version: {}",
        codex.id,
        codex.name,
        codex.storage.root_folder().to_str().unwrap(),
        codex.version().as_str(),
        codex.storage.version().as_str(),
        codex.storage.sqlite().unwrap().version().as_str()
    );

    EngineResponse::CodexOpened {
        id: codex.id,
        name: codex.name,
    }
}

#[aetherium_macros::optional_ml]
pub async fn handler_sync_live<P: AsRef<Path>>(
    codex_path: P,
    on_progress: &(dyn Fn(EngineEvent) + Send + Sync),
) -> EngineResponse {
    // TODO: Add Logging on each operation done in the codex, added, updated or deleted

    let (codex, config) = match utils::open_codex_and_config(codex_path) {
        Ok(v) => v,
        Err(e) => {
            return EngineResponse::Error {
                message: e.message().to_string(),
            };
        }
    };

    let sql_client = match codex.storage.sqlite() {
        Ok(s) => s,
        Err(e) => {
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    // FIX: file_to_embed needs to be mutex when the sync becomes an async method
    // For now this works, since storage::sync is not an async method
    let file_to_embed = match utils::codex_sync_helper(&codex, on_progress) {
        Ok(v) => v,
        Err(e) => {
            return EngineResponse::Error {
                message: e.message(),
            };
        }
    };

    if !__ml_available {
        on_progress(EngineEvent::MLUnavailable);
        return EngineResponse::PartialSynced;
    }
    for file_id in file_to_embed {
        on_progress(EngineEvent::Sync(SyncProgress::Embedding {
            file_id: file_id.clone(),
        }));

        match utils::embed_file_helper(file_id, &codex, &sql_client, &config).await {
            Ok(_) => {}
            Err(e) => {
                return EngineResponse::Error {
                    message: e.message().to_string(),
                };
            }
        }
    }

    EngineResponse::Synced
}

pub async fn handler_add_file<P: AsRef<Path>>(
    codex_path: P,
    file_path: P,
    file_name: Option<String>,
) -> EngineResponse {
    handler_add_file_live(codex_path, file_path, file_name, &|_| {}).await
}

#[aetherium_macros::optional_ml]
pub async fn handler_add_file_live<P: AsRef<Path>>(
    codex_path: P,
    file_path: P,
    file_name: Option<String>,
    on_progress: &(dyn Fn(EngineEvent) + Send + Sync),
) -> EngineResponse {
    let (codex, codexconfig) = match utils::open_codex_and_config(codex_path) {
        Ok(v) => v,
        Err(e) => {
            return EngineResponse::Error {
                message: e.message(),
            };
        }
    };

    let sqlite_client = match codex.storage.sqlite() {
        Ok(sq) => sq,
        Err(e) => {
            return EngineResponse::Error {
                message: e.message().to_string(),
            };
        }
    };

    let added_file = match codex.add_file(
        &file_path.as_ref().to_path_buf(),
        file_name,
        codexconfig.settings.write_chunk_size,
    ) {
        Ok(ad) => ad,
        Err(e) => {
            error!(
                "Got Error Adding File in Codex id: {}, name: {}, file path: {}",
                codex.id,
                codex.name,
                file_path.as_ref().to_string_lossy().to_string()
            );
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    if !__ml_available {
        on_progress(EngineEvent::MLUnavailable)
    } else {
        match embed_file_helper(
            added_file.file_id.clone(),
            &codex,
            sqlite_client,
            &codexconfig,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                error!("Got Error while embedding file");
                return EngineResponse::Error {
                    message: e.message().to_string(),
                };
            }
        }
    }

    EngineResponse::FileAdded {
        file_id: added_file.file_id,
        hash: added_file.file_hash.to_hex().to_string(),
    }
}

#[aetherium_macros::optional_ml]
pub async fn handler_codex_sync<P: AsRef<Path>>(codex_path: P) -> EngineResponse {
    handler_sync_live(codex_path, &|_| {}).await
}

pub fn handler_codex_list_files<P: AsRef<Path>>(codex_path: P) -> EngineResponse {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    let sqlite_conn = match codex.storage.sqlite() {
        Err(err) => {
            error!(
                "Could not open sqlite store for codex: Details -> Id: {}, name: {}, error: {}",
                codex.id,
                codex.name,
                err.message()
            );
            return EngineResponse::Error {
                message: err.message().into(),
            };
        }
        Ok(conn) => conn,
    };

    match sqlite_conn.get_all_files() {
        Err(e) => {
            error!(
                "Could not retrieve all files, please check it, codex id: {}, name: {}, error: {}",
                codex.id,
                codex.name,
                e.message()
            );
            EngineResponse::Error {
                message: e.message().into(),
            }
        }
        Ok(list) => EngineResponse::FileList {
            files: list
                .iter()
                .map(|f| -> FileDetail { f.into() })
                .collect::<Vec<FileDetail>>(),
        },
    }
}

pub fn handler_codex_delete_file<P: AsRef<Path>>(codex_path: P, file_id: String) -> EngineResponse {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    match codex.delete_file(file_id.clone()) {
        Ok(_) => {
            info!(
                "File {} deleted from the codex id: {}, name: {}",
                file_id, codex.id, codex.name
            );
            EngineResponse::FileDeleted
        }
        Err(e) => {
            error!(
                "File {} failed to be deleted from the codex id: {}, name: {}, error: {}",
                file_id,
                codex.id,
                codex.name,
                e.message()
            );
            EngineResponse::Error {
                message: e.message().into(),
            }
        }
    }
}

pub fn handler_get_config<P: AsRef<Path>>(codex_path: P, key: String) -> EngineResponse {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    match codex.get_config(key) {
        Ok(cf) => EngineResponse::GotConfig {
            value: Some(cf.to_string()),
        },

        Err(StorageError::NotFound(_)) => EngineResponse::GotConfig { value: None },

        Err(e) => EngineResponse::Error {
            message: e.message().into(),
        },
    }
}

pub fn handler_set_config<P: AsRef<Path>>(codex_path: P, key: &str, value: &str) -> EngineResponse {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    match codex.change_config(key, value) {
        Ok(_) => EngineResponse::SettedConfig {
            key: key.into(),
            val: value.into(),
        },
        Err(e) => EngineResponse::Error {
            message: e.message().into(),
        },
    }
}

#[aetherium_macros::optional_ml]
pub async fn handler_get_files<P: AsRef<Path>>(
    codex_path: P,
    query: String,
    query_type: String,
    top_k: usize,
) -> EngineResponse {
    let codex = match utils::open_codex(codex_path) {
        Ok(c) => c,
        Err(e) => {
            return EngineResponse::Error {
                message: e.message().to_string(),
            };
        }
    };

    let sqlite_client = match codex.storage.sqlite() {
        Ok(c) => c,
        Err(e) => {
            return EngineResponse::Error {
                message: e.message().to_string(),
            };
        }
    };
    let qt = match SearchType::parse_str(&query_type) {
        Ok(b) => b,
        Err(e) => {
            return EngineResponse::Error {
                message: e.message(),
            };
        }
    };

    match qt {
        SearchType::Semantic | SearchType::Mix => {
            if !__ml_available {
                return EngineResponse::Error {
                    message: "ml server unavailable...".to_string(),
                };
            }
            let query_embed = grpc_ops::handler_ml_get_query_embed(query.clone()).await;

            match query_embed {
                Ok(embed) => {
                    let mut results =
                        match sqlite_client.find_similar_embedding(embed.vector, top_k) {
                            Ok(r) => r,
                            Err(e) => {
                                return EngineResponse::Error {
                                    message: e.message().to_string(),
                                };
                            }
                        };
                    results.sort_by(|f1, f2| f2.distance.total_cmp(&f1.distance));
                    EngineResponse::SearchResults {
                        results: results
                            .into_iter()
                            .map(|f| {
                                (
                                    SearchMatchedDetails {
                                        file_id: f.doc_id.clone(),
                                        chunk_id: f.chunk_id,
                                        file_name: f.file_name,
                                        distance: f.distance,
                                        cluster: f.cluster,
                                    },
                                    codex
                                        .storage
                                        .read_file_delimited(f.doc_id, f.start_char, f.end_char)
                                        .unwrap_or("Error Reading the File".to_string()),
                                )
                            })
                            .collect::<Vec<(_, _)>>(),
                    }
                }
                Err(e) => EngineResponse::Error {
                    message: e.message(),
                },
            }
        }

        SearchType::Lexical => todo!(),
    }
}

#[aetherium_macros::require_ml]
pub async fn handler_cluster<P: AsRef<Path>>(
    codex_path: P,
    on_progress: &(dyn Fn(EngineEvent) + Send + Sync),
) -> EngineResponse {
    let (codex, config) = match utils::open_codex_and_config(codex_path) {
        Ok(c) => c,
        Err(e) => return e.into(),
    };

    let sql_client = match codex.storage.sqlite() {
        Ok(s) => s,
        Err(e) => engine_err!(e.message()),
    };

    let ml_config = match MLConfig::load() {
        Ok(s) => s,
        Err(e) => engine_err!(e.message()),
    };

    on_progress(EngineEvent::OperationStarted);

    if let Err(e) = sql_client.clear_clusters() {
        engine_err!(e.message());
    };

    let corpus =
        match TFIDFCorpus::build_from_storage(&codex.storage, config.settings.read_chunk_size) {
            Ok(c) => c,
            Err(e) => engine_err!(e.message()),
        };

    let chunks = match ml_config.dim_reduction_model {
        DimensionsReductionModel::UMAP | DimensionsReductionModel::NONE => {
            match sql_client.get_all_embeddings() {
                Ok(embeddings) => embeddings
                    .into_iter()
                    .map(|e| ClusterChunkInput {
                        chunk_id: e.chunk_id,
                        doc_id: e.doc_id,
                        vector: VectorInput::Embedding(e.embedding),
                    })
                    .collect::<Vec<_>>(),

                Err(e) => engine_err!(e.message()),
            }
        }

        DimensionsReductionModel::LDA | DimensionsReductionModel::LSA => {
            let chunk_positions = match sql_client.get_all_chunks() {
                Ok(c) => c,
                Err(e) => engine_err!(e.message()),
            };

            let vocab: Vec<String> = corpus.vocabulary();
            let mut result = Vec::new();

            for chunk in chunk_positions {
                let text = match codex.storage.read_file_delimited(
                    chunk.doc_id.clone(),
                    chunk.start_char,
                    chunk.end_char,
                ) {
                    Ok(t) => t,
                    Err(e) => engine_err!(e.message()),
                };

                let tf = TFIDFCorpus::compute_tf_from_str(&text);
                let vector: Vec<f32> = vocab
                    .iter()
                    .map(|term| *tf.get(term).unwrap_or(&0) as f32)
                    .collect();

                result.push(ClusterChunkInput {
                    chunk_id: chunk.id,
                    doc_id: chunk.doc_id,
                    vector: VectorInput::TFIDF(vector),
                });
            }
            result
        }
    };

    on_progress(EngineEvent::Clustering);

    let ml_response = match grpc_ops::handler_ml_cluster(chunks, &ml_config).await {
        Ok(r) => r,
        Err(e) => engine_err!(e.message()),
    };

    let mut cluster_docs: HashMap<i32, Vec<String>> = HashMap::new();

    for assignment in &ml_response.assignments {
        cluster_docs
            .entry(assignment.cluster_id)
            .or_default()
            .push(assignment.chunk_id.clone());
    }

    let mut cluster_names: Vec<(String, i32)> = Vec::new();

    let assignments: Vec<(String, i32)> = ml_response
        .assignments
        .iter()
        .map(|a| (a.chunk_id.clone(), a.cluster_id))
        .collect();

    let chunk_to_doc: HashMap<String, String> = sql_client
        .get_all_chunks()
        .unwrap_or_default()
        .into_iter()
        .map(|c| (c.id, c.doc_id))
        .collect();

    for (cluster_id, chunk_ids) in &cluster_docs {
        let name = if *cluster_id == -1 {
            "Outlier".to_string()
        } else {
            let doc_ids: Vec<&str> = chunk_ids
                .iter()
                .filter_map(|cid| chunk_to_doc.get(cid).map(|s| s.as_str()))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            let top_terms = corpus.top_terms_for_docs(&doc_ids, 3);

            top_terms
                .iter()
                .map(|(term, _)| term.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };
        if let Err(e) = sql_client.write_cluster_info(*cluster_id, &name) {
            engine_err!(e.message())
        }

        cluster_names.push((name.clone(), *cluster_id));
    }

    if let Err(e) = sql_client.write_cluster_chunks(&assignments) {
        engine_err!(e.message())
    }

    on_progress(EngineEvent::OperationFinished);

    EngineResponse::Clustered {
        clusters_found: ml_response.n_clusters as u32,
        unique_clusters: cluster_names,
    }
}

pub fn handler_ml_config_setup() -> EngineResponse {
    let ml_config = MLConfig::default();

    let config_dir = match aetherium_core::CONFIG_DIR.as_ref() {
        Some(d) => d,
        None => {
            return EngineResponse::Error {
                message: "config dir not found".into(),
            };
        }
    };

    if let Err(e) = std::fs::create_dir_all(config_dir) {
        return EngineResponse::Error {
            message: e.to_string(),
        };
    }

    let path = config_dir.join("ml.toml");

    if path.exists() {
        return EngineResponse::Error {
            message: "ml.toml already exists, delete it first to regenerate".into(),
        };
    }

    let contents = match toml::to_string_pretty(&ml_config) {
        Ok(c) => c,
        Err(e) => {
            return EngineResponse::Error {
                message: e.to_string(),
            };
        }
    };

    if let Err(e) = std::fs::write(&path, contents) {
        return EngineResponse::Error {
            message: e.to_string(),
        };
    }

    EngineResponse::MLConfigSetupSuccessful {
        path: path.to_string_lossy().to_string(),
    }
}

pub fn handler_list_files_with_clusters<P: AsRef<Path>>(codex_path: P) -> EngineResponse {
    let codex = match utils::open_codex(codex_path) {
        Err(e) => engine_err!(e.message()),
        Ok(c) => c,
    };

    let sql_client = match codex.storage.sqlite() {
        Err(e) => engine_err!(e.message()),
        Ok(s) => s,
    };

    let files = match sql_client.list_files_with_top_clusters() {
        Ok(f) => f,
        Err(e) => engine_err!(e.message()),
    };

    EngineResponse::FileListWithClusters {
        files: files
            .into_iter()
            .map(|f| FileDetailWithCluster {
                id: f.id,
                name: f.name,
                extension: f.extension,
                created_at: f.created_at,
                cluster_name: f.cluster_name,
                top_cluster_pct: f.top_cluster_pct,
            })
            .collect(),
    }
}

pub fn handler_get_cluster_info<P: AsRef<Path>>(codex_path: P) -> EngineResponse {
    let codex = match utils::open_codex(codex_path) {
        Err(e) => engine_err!(e.message()),
        Ok(c) => c,
    };

    let sql_client = match codex.storage.sqlite() {
        Err(e) => engine_err!(e.message()),
        Ok(s) => s,
    };

    let stats = match sql_client.basic_describe_clusters() {
        Err(e) => engine_err!(e.message()),
        Ok(s) => s,
    };

    let ml_config = MLConfig::load().unwrap_or_default();

    EngineResponse::ClusterStats {
        model_info: crate::types::ClusterModelInfo {
            name: ml_config.cluster_model.parse().to_string(),
            dimension_reduction_model: ml_config.dim_reduction_model.parse().to_string(),
            dims: ml_config.dims as usize,
            reduced_to: match ml_config.dim_reduction_model {
                DimensionsReductionModel::LDA => ml_config.lda.n_components,
                DimensionsReductionModel::LSA => ml_config.lsa.n_components,
                DimensionsReductionModel::UMAP => ml_config.umap.n_components,
                DimensionsReductionModel::NONE => ml_config.dims as usize,
            },
        },

        stats: stats.into_iter().map(|s| s.into()).collect::<Vec<_>>(),
    }
}
