use std::{env, path, sync::LazyLock};

pub mod cacher;
pub mod codex;
pub mod metadata;
pub mod storage;
pub mod tfidf;

pub static CURRENT_DIR: LazyLock<Option<std::path::PathBuf>> = LazyLock::new(|| {
    let pare_path = env::current_dir().ok();
    path::absolute(&pare_path?).ok()
});
