use std::{
    fs::{self, File},
    io::{BufReader, Read, Write},
    path::Path,
};

use blake3::Hash;

use crate::{codex::codex_config::CodexConfig, storage::CODEX_FILE};

/// write_to_file
/// returns anyhow::Result<tuple(blake3::Hash,String)>
/// writes file in the foldername given (filename) returns the finalized hash and hash as hex
pub fn write_to_file<R>(file: &mut File, data: R, byte: usize) -> anyhow::Result<(Hash, String)>
where
    R: Read,
{
    let mut reader = BufReader::new(data);
    let mut buffer = vec![0; byte];
    let mut hasher = blake3::Hasher::new();
    let result = || -> anyhow::Result<()> {
        loop {
            let left = reader.read(&mut buffer)?;
            if left == 0 {
                break;
            }
            hasher.update(&buffer[..left]);
            file.write_all(&buffer[..left])?;
        }

        file.flush()?;
        file.sync_all()?;

        Ok(())
    }();

    result?;

    let file_hash = hasher.finalize();
    let file_id = file_hash.to_hex().to_string();
    Ok((file_hash, file_id))
}

pub fn read_codex_config(root_folder: &Path) -> anyhow::Result<CodexConfig> {
    let read_data = fs::read_to_string(root_folder.join(CODEX_FILE))?;
    let data: CodexConfig = toml::from_str(read_data.as_str())?;

    Ok(data)
}
