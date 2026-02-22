pub mod layout;
pub mod v1;

pub enum SqliteStoreVersion {
    V1,
}

impl SqliteStoreVersion {
    pub fn parse(version: &str) -> Self {
        match version {
            "v1" => SqliteStoreVersion::V1,
            _ => SqliteStoreVersion::V1,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SqliteStoreVersion::V1 => "v1",
        }
    }
}
