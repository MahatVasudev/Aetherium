use crate::storage::versions::v1::StorageV1;

pub mod layout;
pub mod v1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageVersion {
    V1,
}

impl StorageVersion {
    pub fn parse(version: &str) -> Option<StorageVersion> {
        match version {
            "v1" => Some(StorageVersion::V1),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            StorageVersion::V1 => "v1",
        }
    }
}
pub fn load_storageversion(version: &StorageVersion) -> Box<dyn layout::StorageLayout> {
    match version {
        StorageVersion::V1 => Box::new(StorageV1),
    }
}
