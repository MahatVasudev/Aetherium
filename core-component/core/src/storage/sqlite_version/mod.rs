pub mod layout;
pub mod v1;

pub enum SqliteStoreVersion {
    V1,
}

impl SqliteStoreVersion {
    pub fn latest() -> Self {
        SqliteStoreVersion::V1
    }
    pub fn parse(version: &str) -> Option<Self> {
        match version {
            "v1" => Some(SqliteStoreVersion::V1),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SqliteStoreVersion::V1 => "v1",
        }
    }
}
