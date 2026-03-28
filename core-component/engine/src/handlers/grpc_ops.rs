use aetherium_core::ml_server::config::MLConfig;
use tonic::IntoRequest;

use crate::{
    EngineResponse,
    error::EngineError,
    ml_client::{
        self,
        aetherium_ml::{EmbedBatchResponse, EmbedQueryResponse},
    },
    types::DocTextChunk,
};

pub async fn handler_ml_health() -> EngineResponse {
    let mlconfig = match MLConfig::load() {
        Ok(config) => config,
        Err(e) => {
            return EngineResponse::Error {
                message: e.message(),
            };
        }
    };

    let mut connect = match ml_client::MLClient::connect(&mlconfig).await {
        Ok(connect) => connect,
        Err(e) => return e.into(),
    };

    let health_status = match connect.health().await {
        Ok(status) => status,
        Err(e) => return e.into(),
    };

    EngineResponse::MLHealth {
        status: health_status.status,
        version: health_status.version,
        model: health_status.model,
        dims: health_status.dims,
    }
}

pub async fn handler_ml_get_query_embed(query: String) -> Result<EmbedQueryResponse, EngineError> {
    let mlconfig = match MLConfig::load() {
        Ok(config) => config,
        Err(e) => return Err(EngineError::Corrupt(e.message())),
    };
    let mut connect = match ml_client::MLClient::connect(&mlconfig).await {
        Ok(c) => c,
        Err(e) => return Err(e.into()),
    };

    let query_embed = match connect.embed_query(query).await {
        Ok(embedded) => embedded,
        Err(e) => return Err(e.into()),
    };

    Ok(query_embed)
}

pub async fn handler_ml_get_batch_embed(
    batches: Vec<DocTextChunk>,
) -> Result<EmbedBatchResponse, EngineError> {
    let mlconfig = match MLConfig::load() {
        Ok(config) => config,
        Err(e) => return Err(EngineError::Corrupt(e.message())),
    };

    let mut connect = match ml_client::MLClient::connect(&mlconfig).await {
        Ok(c) => c,
        Err(e) => return Err(e.into()),
    };

    let batch_embed = match connect.embed_batch(batches).await {
        Ok(embedded) => embedded,
        Err(e) => return Err(e),
    };

    Ok(batch_embed)
}
