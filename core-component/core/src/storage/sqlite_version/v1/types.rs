pub struct FileInSQL {
    pub id: String,
    pub name: String,
    pub hash: String,
    pub extension: String,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub indexed_at: Option<String>,
    pub embedded_at: Option<String>,
}

pub struct Info {
    codex_version: String,
    storage_version: String,
    sqlite_version: String,
}

pub struct TriggerTables {
    pub table_name: String,
    pub col: String,
}
