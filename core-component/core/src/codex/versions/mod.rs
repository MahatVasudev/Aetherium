pub mod v1;

use std::{
    io::Read,
    path::{Path, PathBuf},
};

use crate::{
    codex::{Codex, file_reading::FileAddedResponse, versions::v1::CodexV1},
    storage::versions::StorageVersion,
};

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

pub trait CodexLayout {
    fn version(&self) -> CodexVersion;
    fn build(&self, root_folder: &Path, storage_version: StorageVersion) -> anyhow::Result<Codex>;
    fn first_codex_content(
        &self,
        codex_name: &str,
        generated_id: &str,
        storage_version: StorageVersion,
    ) -> String;
    fn add_file(
        &self,
        codex: &Codex,
        from_filename: &PathBuf,
        byte: usize,
    ) -> anyhow::Result<FileAddedResponse>;
    fn search_files(&self, query: &str) -> Vec<PathBuf>;
    fn read_file(&self, file_name: &str) -> String;
    fn supported_storage(&self) -> &'static [StorageVersion];
}
