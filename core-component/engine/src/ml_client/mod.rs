use crate::{
    error::EngineError,
    ml_client::aetherium_ml::{
        ChunkVector, ClusterRequest, ClusterResponse, EmbedBatchRequest, EmbedBatchResponse,
        EmbedQueryRequest, EmbedQueryResponse, EmbeddingVector, HdbscanConfig, HealthRequest,
        HealthResponse, LdaConfig, LsaConfig, NoneReduceConfig, TextChunk, TfidfVector, UmapConfig,
        aetherium_ml_service_client::AetheriumMlServiceClient, chunk_vector, cluster_request,
    },
    types::{ClusterChunkInput, DocTextChunk, VectorInput},
};
use aetherium_core::ml_server::config::{DimensionsReductionModel, MLConfig};
pub struct MLClient {
    client: AetheriumMlServiceClient<tonic::transport::Channel>,
}

impl MLClient {
    pub async fn connect(config: &MLConfig) -> Result<Self, EngineError> {
        let channel = tonic::transport::Channel::from_shared(config.address())
            .map_err(|e| EngineError::InvalidUri(e))?
            .connect()
            .await
            .map_err(|e| EngineError::ConnectionError(e))?;

        let client = AetheriumMlServiceClient::new(channel)
            .max_decoding_message_size(config.message_size_mb * 1024 * 1024) // 64MB
            .max_encoding_message_size(config.message_size_mb * 1024 * 1024); // 64MB

        Ok(Self { client })
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

    pub async fn cluster(
        &mut self,
        chunks: Vec<ClusterChunkInput>,
        ml_config: &MLConfig,
    ) -> Result<ClusterResponse, EngineError> {
        let total_chunks = chunks.len();
        let chunk_vectors: Vec<ChunkVector> = chunks
            .into_iter()
            .map(|c| ChunkVector {
                chunk_id: c.chunk_id,
                doc_id: c.doc_id,
                vector: Some(match c.vector {
                    VectorInput::Embedding(v) => {
                        chunk_vector::Vector::Embedding(EmbeddingVector { values: v })
                    }
                    VectorInput::TFIDF(v) => chunk_vector::Vector::Tfidf(TfidfVector { values: v }),
                }),
            })
            .collect();

        let reducer_config = match ml_config.dim_reduction_model {
            DimensionsReductionModel::UMAP => {
                Some(cluster_request::ReducerConfig::UmapConfig(UmapConfig {
                    n_components: ml_config.umap.n_components as u32,
                    n_neighbors: ml_config.umap.n_neighbors as u32,
                    min_distance: ml_config.umap.min_distance,
                    metric: ml_config.umap.metric.clone(),
                }))
            }
            DimensionsReductionModel::LDA => {
                Some(cluster_request::ReducerConfig::LdaConfig(LdaConfig {
                    n_components: ml_config.lda.n_components as u32,
                    max_iter: ml_config.lda.max_iter as u32,
                    learning_method: ml_config.lda.learning_method.clone(),
                }))
            }
            DimensionsReductionModel::LSA => {
                Some(cluster_request::ReducerConfig::LsaConfig(LsaConfig {
                    n_components: ml_config.lsa.n_components as u32,
                }))
            }
            DimensionsReductionModel::NONE => Some(cluster_request::ReducerConfig::NoneConfig(
                NoneReduceConfig {},
            )),
        };

        // FIX: Add implementation of Kmeans and Dbscan
        //
        //
        let adaptive_min_cluster_size = (total_chunks / 10).max(3).min(30);

        let cluster_config = Some(cluster_request::ClusterConfig::Hdbconfig(HdbscanConfig {
            min_cluster_size: adaptive_min_cluster_size as u32,
            min_samples: ml_config.hdbscan.min_samples.map(|v| v as u32),
            metric: ml_config.hdbscan.metric.clone(),
        }));

        let response = self
            .client
            .cluster(ClusterRequest {
                chunks: chunk_vectors,
                cluster_method: ml_config.cluster_model.parse().to_string(),
                reducer_method: ml_config.dim_reduction_model.parse().to_string(),
                reducer_config,
                cluster_config,
            })
            .await?
            .into_inner();

        Ok(response)
    }
}

pub mod aetherium_ml {
    tonic::include_proto!("aetherium_ml");
}
