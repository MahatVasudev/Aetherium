use std::{
    fs,
    io::{BufReader, Read, Write},
    path::PathBuf,
};

use blake3::Hash;

use crate::codex::{Codex, utils};

impl Codex {
    pub fn add_file<R>(
        &self,
        data: R,
        byte: usize,
        filename: &str,
    ) -> anyhow::Result<FileAddedResponse>
    where
        R: Read,
    {
        // WARN: Incomplete Implementation (works for now)
        let temp_folder = utils::create_temp_if_not_exists(&self.root_folder)?;
        let temp_filename = temp_folder.join(filename);
        let (file_hash, file_id) = match utils::write_to_file(&temp_filename, data, byte) {
            Ok(result) => result,
            Err(result) => {
                fs::remove_file(&temp_filename)?;
                return Err(result);
            }
        };

        let final_filename = self.data_folder.join(&file_id);
        fs::rename(&temp_filename, &final_filename)?;
        Ok(FileAddedResponse {
            file_path: final_filename,
            file_id,
            file_hash,
        })
    }
    pub fn search_files(&self, query: &str) -> Vec<PathBuf> {
        unimplemented!()
    }
    pub fn read_file(&self, file_name: &str) -> String {
        unimplemented!()
    }
}

pub struct FileAddedResponse {
    file_path: PathBuf,
    file_id: String,
    file_hash: Hash,
}

#[cfg(test)]
mod testing {
    use std::{fs::File, path::Path};

    use super::*;
    #[test]
    fn writing_file_ok() {
        let codex = Codex::open("/home/clyde/Documents/first-knowledge").unwrap();
        let raw_filename = Path::new("/home/clyde/Downloads/sml_importance .pdf");
        let filename = raw_filename.file_name().unwrap().to_str().unwrap();
        let file_to_w = File::open(&raw_filename).unwrap();
        let buffersize: usize = 512;

        let written = codex.add_file(file_to_w, buffersize, filename);

        assert!(written.is_ok())
    }
}
