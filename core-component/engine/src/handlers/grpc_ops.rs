use aetherium_core::ml_server::config::MLConfig;

use crate::{EngineResponse, ml_client};

pub async fn handler_ml_health() -> EngineResponse {
    let mlconfig = match MLConfig::load() {
        Ok(config) => config,
        Err(e) => {
            return EngineResponse::Error {
                message: e.message(),
            };
        }
    };

    let connect = match ml_client::MLClient::connect(&mlconfig).await {
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
