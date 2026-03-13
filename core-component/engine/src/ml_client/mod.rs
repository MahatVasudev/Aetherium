use crate::{
    error::EngineError,
    ml_client::aetherium_ml::{
        HealthRequest, HealthResponse, aetherium_ml_service_client::AetheriumMlServiceClient,
    },
};
use aetherium_core::ml_server::config::MLConfig;
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
}

pub mod aetherium_ml {
    tonic::include_proto!("aetherium_ml");
}
