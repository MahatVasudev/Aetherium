pub mod v1;

use crate::codex::{layout::CodexLayout, versions::v1::CodexV1};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexVersion {
    V1,
    V2,
}

impl CodexVersion {
    pub fn parse(version: &str) -> Option<Self> {
        match version {
            "v1.0.0" => Some(Self::V1),
            "v2.0.0" => Some(Self::V2),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1 => "v1.0.0",
            Self::V2 => "v2.0.0",
        }
    }
}

pub fn layout_for(version: CodexVersion) -> Box<dyn CodexLayout> {
    match version {
        CodexVersion::V1 => Box::new(CodexV1),
        CodexVersion::V2 => Box::new(CodexV1),
    }
}
