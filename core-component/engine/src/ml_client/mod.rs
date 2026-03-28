use crate::{
    error::EngineError,
    ml_client::aetherium_ml::{
        EmbedBatchRequest, EmbedBatchResponse, EmbedQueryRequest, EmbedQueryResponse,
        HealthRequest, HealthResponse, TextChunk,
        aetherium_ml_service_client::AetheriumMlServiceClient,
    },
    types::DocTextChunk,
};
use aetherium_core::{ml_server::config::MLConfig, tfidf::sentence_chunker::SentenceChunks};
pub struct MLClient {
    client: AetheriumMlServiceClient<tonic::transport::Channel>,
}

impl MLClient {
    pub async fn connect(config: &MLConfig) -> Result<Self, EngineError> {
        match AetheriumMlServiceClient::connect(config.address()).await {
            Ok(client) => Ok(Self { client }),
            Err(e) => Err(EngineError::ConnectionError(e)),
        }
    }

    pub async fn health(&mut self) -> Result<HealthResponse, EngineError> {
        Ok(self.client.health(HealthRequest {}).await?.into_inner())
    }

    pub async fn embed_query(&mut self, query: String) -> Result<EmbedQueryResponse, EngineError> {
        Ok(self
            .client
            .embed_query(EmbedQueryRequest { query })
            .await?
            .into_inner())
    }

    pub async fn embed_batch(
        &mut self,
        sentence: Vec<DocTextChunk>,
    ) -> Result<EmbedBatchResponse, EngineError> {
        Ok(self
            .client
            .embed_batch(EmbedBatchRequest {
                chunks: sentence
                    .into_iter()
                    .map(|s| TextChunk {
                        doc_id: s.doc_id,
                        chunk_id: s.chunk_id,
                        chunk_index: s.file_index as i32,
                        text: s.text,
                    })
                    .collect(),
            })
            .await?
            .into_inner())
    }
}

pub mod aetherium_ml {
    tonic::include_proto!("aetherium_ml");
}
