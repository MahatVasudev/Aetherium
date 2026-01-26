pub mod v1;

use std::{
    io::Read,
    path::{Path, PathBuf},
};

use crate::codex::{Codex, file_reading::FileAddedResponse, versions::v1::CodexV1};

#[derive(Debug, Clone, Copy)]
pub enum CodexVersion {
    V1,
}

impl CodexVersion {
    pub fn parse(version: &str) -> Option<Self> {
        match version {
            "v1.0.0" => Some(Self::V1),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1 => "v1.0.0",
        }
    }
}

pub fn layout_for(version: CodexVersion) -> &'static dyn CodexLayout {
    match version {
        _ => &CodexV1,
    }
}

pub trait CodexLayout {
    fn build(&self, root_folder: &Path) -> anyhow::Result<Codex>;
    fn open(&self, root_folder: PathBuf) -> anyhow::Result<Codex>;
    fn write_first_codex(
        &self,
        foldername: &Path,
        codex_name: &String,
        generated_id: String,
    ) -> anyhow::Result<()>;
    fn validate_codex_at(&self, root_folder: &Path) -> bool;
    fn is_codex(&self, root_folder: &Path) -> bool;

    fn add_file(
        &self,
        codex: &Codex,
        data: &mut dyn Read,
        byte: usize,
        filename: &str,
    ) -> anyhow::Result<FileAddedResponse>;
    fn validate(&self, codex: &Codex) -> bool;
    fn search_files(&self, query: &str) -> Vec<PathBuf>;
    fn read_file(&self, file_name: &str) -> String;
}
