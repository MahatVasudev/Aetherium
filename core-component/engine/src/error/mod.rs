use crate::EngineResponse;
use std::fmt::Display;

#[derive(Debug)]
pub enum EngineError {
    MLServerUnavailable(String),
    Corrupt(String),
    ConnectionError(tonic::transport::Error),
    GRPCError(tonic::Status),
    SearchModeNotFound(String),
    CodexCouldNotOpen(String),
    SyncFail(String),
    InvalidUri(tonic::codegen::http::uri::InvalidUri),
}

impl EngineError {
    pub fn message(&self) -> String {
        match self {
            Self::MLServerUnavailable(v) => format!("ML Server Unavailable: {v}"),
            Self::Corrupt(v) => format!("Something has been corrupted: {v}"),
            Self::ConnectionError(v) => format!("GRPC connection error: {}", v.to_string()),
            Self::GRPCError(v) => format!("GRPC status error: {}", v.to_string()),
            Self::SearchModeNotFound(v) => {
                format!("Search Mode Type Given Could not be found: {}", v)
            }
            Self::CodexCouldNotOpen(v) => format!("Codex could not be open: {}", v),
            Self::SyncFail(v) => format!("Codex Sync Failed: {}", v),
            Self::InvalidUri(v) => format!("Invalid Uri Recieved: {}", v),
        }
    }
}

impl Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => write!(f, "{}", self.message()),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<EngineError> for EngineResponse {
    fn from(value: EngineError) -> Self {
        match value {
            EngineError::MLServerUnavailable(v) => Self::MLUnavailable(v),
            _ => Self::Error {
                message: value.message(),
            },
        }
    }
}

impl From<tonic::transport::Error> for EngineError {
    fn from(value: tonic::transport::Error) -> Self {
        Self::ConnectionError(value)
    }
}

impl From<tonic::Status> for EngineError {
    fn from(value: tonic::Status) -> Self {
        Self::GRPCError(value)
    }
}
