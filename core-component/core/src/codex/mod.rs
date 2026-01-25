pub mod file_reading;
pub mod utils;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

const CODEX_FILE: &str = "codex.toml";
const DATA_FOLDER: &str = "data";
const INDEXED_FOLDER: &str = "indexed";
const DATABASE_FOLDER: &str = "database";

struct Codex {
    pub root_folder: PathBuf,
    pub data_folder: PathBuf,
    pub indexed_folder: PathBuf,
    pub database_folder: PathBuf,
    pub config_file: PathBuf,
}

impl Codex {
    pub fn build(root_folder: &str) -> anyhow::Result<Codex> {
        // WARN: Incomplete Implementation (works for now)
        if Codex::is_codex(root_folder) {
            anyhow::bail!("This folder is already an codex")
        }

        fs::create_dir(&root_folder)
            .with_context(|| format!("failed to create root dir {:?}", root_folder))?;
        let folder_name = Path::new(&root_folder).to_path_buf();
        let result = (|| -> anyhow::Result<()> {
            fs::create_dir(folder_name.join(DATA_FOLDER))?;
            fs::create_dir(folder_name.join(INDEXED_FOLDER))?;
            fs::create_dir(folder_name.join(DATABASE_FOLDER))?;
            Codex::write_first_codex(&folder_name)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(&folder_name);
        }

        result?;
        Ok(Codex {
            data_folder: folder_name.join(DATA_FOLDER),
            indexed_folder: folder_name.join(INDEXED_FOLDER),
            database_folder: folder_name.join(DATABASE_FOLDER),
            config_file: folder_name.join(CODEX_FILE),
            root_folder: folder_name,
        })
    }
    pub fn open(root_folder: &str) -> anyhow::Result<Codex> {
        // WARN: Incomplete Implementation (works for now)
        if !Codex::is_codex(root_folder) {
            anyhow::bail!("This is not a codex")
        }

        // Create the folders if they are missing

        let folder_name = Path::new(&root_folder).to_path_buf();
        let result = (|| -> anyhow::Result<()> {
            fs::create_dir_all(folder_name.join(DATA_FOLDER))?;
            fs::create_dir_all(folder_name.join(INDEXED_FOLDER))?;
            fs::create_dir_all(folder_name.join(DATABASE_FOLDER))?;
            Ok(())
        })();

        result?;

        Ok(Codex {
            data_folder: folder_name.join(DATA_FOLDER),
            indexed_folder: folder_name.join(INDEXED_FOLDER),
            database_folder: folder_name.join(DATABASE_FOLDER),
            config_file: folder_name.join(CODEX_FILE),
            root_folder: folder_name,
        })
    }
    fn write_first_codex(foldername: &PathBuf) -> anyhow::Result<()> {
        let codex_content = "\
[version]
codex = \"v1\"

                             ";
        fs::write(foldername.join(CODEX_FILE), codex_content)?;

        Ok(())
    }
    pub fn is_codex(root_folder: &str) -> bool {
        Path::new(root_folder).join(CODEX_FILE).exists()
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    use tempfile::{Builder, tempdir};
    #[test]
    fn it_should_work() {
        let temp = tempdir().unwrap();
        let foldername = temp.path().join("my_codex");
        let _ = Codex::build(foldername.to_str().unwrap()).expect("Codex should have worked");

        assert!(foldername.join(CODEX_FILE).exists());
        assert!(foldername.join(DATA_FOLDER).exists());
        assert!(foldername.join(INDEXED_FOLDER).exists());
        assert!(foldername.join(DATABASE_FOLDER).exists());
    }

    #[test]
    fn it_should_not_work() {
        let temp = tempdir().unwrap();
        let foldername = temp.path().join("my_codex");
        fs::create_dir(&foldername).unwrap();
        fs::write(foldername.join(CODEX_FILE), "somecontent\nversion 1").unwrap();

        let result = Codex::build(foldername.to_str().unwrap());

        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn permissions_error() {
        let owner_rwx = fs::Permissions::from_mode(0o400);
        let tempdir = Builder::new().permissions(owner_rwx).tempdir().unwrap();

        let foldername = tempdir.path().join("my_codex");

        let result = Codex::build(foldername.to_str().unwrap());
        assert!(result.is_err());
    }
}
