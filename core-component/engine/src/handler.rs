use aetherium_core::codex::Codex;

use crate::{
    Engine, EngineLiveRequest, EngineRequest, EngineResponse,
    handlers::{self, grpc_ops},
    types::EngineEvent,
};

impl Engine {
    pub fn new(codex: Option<Codex>) -> Self {
        Self { codex }
    }

    pub async fn handle_live(
        &self,
        request: EngineLiveRequest,
        on_progress: &(dyn Fn(EngineEvent) + Send + Sync),
    ) -> EngineResponse {
        use EngineLiveRequest::*;

        match request {
            Sync { codex_path } => {
                return handlers::handler_sync_live(codex_path, on_progress).await;
            }

            AddFile {
                codex_path,
                file_path,
                file_name,
            } => {
                return handlers::handler_add_file_live(
                    codex_path,
                    file_path,
                    file_name,
                    on_progress,
                )
                .await;
            }

            Cluster { codex_path } => {
                return handlers::handler_cluster(codex_path, on_progress).await;
            }

            _ => EngineResponse::Error {
                message: "not implemented".into(),
            },
        }
    }
    pub async fn handle(&self, request: EngineRequest) -> EngineResponse {
        use EngineRequest::*;
        match request {
            CreateCodex {
                path,
                codex_version,
                storage_version,
                sqlite_version,
            } => {
                return handlers::handler_create_codex(
                    path,
                    &codex_version,
                    &storage_version,
                    &sqlite_version,
                );
            }
            OpenCodex { path } => return handlers::handler_open_codex(path),
            AddFile {
                codex_path,
                file_path,
                file_name,
            } => return handlers::handler_add_file(codex_path, file_path, file_name).await,

            GetConfig { codex_path, key } => return handlers::handler_get_config(codex_path, key),

            SetConfig {
                codex_path,
                key,
                val,
            } => return handlers::handler_set_config(codex_path, &key, &val),

            Sync { codex_path } => return handlers::handler_codex_sync(codex_path).await,

            ListFiles { codex_path } => return handlers::handler_codex_list_files(codex_path),

            DeleteFile {
                codex_path,
                file_id,
            } => return handlers::handler_codex_delete_file(codex_path, file_id),

            MLHealth => return grpc_ops::handler_ml_health().await,

            SearchFiles {
                codex_path,
                query,
                query_type,
                top_k,
            } => return handlers::handler_get_files(codex_path, query, query_type, top_k).await,

            Cluster { codex_path } => return handlers::handler_cluster(codex_path, &|_| {}).await,

            MLConfigSetup => return handlers::handler_ml_config_setup(),

            ClusterStats { codex_path } => return handlers::handler_get_cluster_info(codex_path),
            ListFileWithClusters { codex_path } => {
                return handlers::handler_list_files_with_clusters(codex_path);
            }

            _ => EngineResponse::Error {
                message: "not yet implemented".into(),
            },
        }
    }
}
