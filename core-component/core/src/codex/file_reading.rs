use std::{fs, io::Read, path::PathBuf};

use blake3::Hash;

use crate::codex::{Codex, CodexLayout, utils};

impl Codex {
    fn add_file<R>(
        &self,
        mut data: R,
        byte: usize,
        filename: &str,
    ) -> anyhow::Result<FileAddedResponse>
    where
        R: Read,
    {
        self.layout().add_file(self, &mut data, byte, filename)
    }
    fn validate(&self) -> bool {
        self.layout().validate(&self)
    }
    fn search_files(&self, query: &str) -> Vec<PathBuf> {
        unimplemented!()
    }
    fn read_file(&self, file_name: &str) -> String {
        unimplemented!()
    }
}

pub struct FileAddedResponse {
    pub file_path: PathBuf,
    pub file_id: String,
    pub file_hash: Hash,
}

#[cfg(test)]
mod testing {
    use std::{fs::File, path::Path};

    use super::*;
    #[test]
    // Checking whether it works on my local machine, for evidence and satisfaction
    fn writing_file_ok() {
        let codex =
            Codex::open(Path::new("/home/clyde/Documents/first-knowledge").to_path_buf()).unwrap();
        let raw_filename = Path::new("/home/clyde/Downloads/sml_importance .pdf");
        let filename = raw_filename.file_name().unwrap().to_str().unwrap();
        let mut file_to_w = File::open(&raw_filename).unwrap();
        let buffersize: usize = 512;

        let written = codex.add_file(file_to_w, buffersize, filename);

        assert!(written.is_ok())
    }

    fn writing_file_ok_2() {
        let codex =
            Codex::open(Path::new("/home/clyde/Documents/first-knowledge").to_path_buf()).unwrap();
        let raw_filename = Path::new("/home/clyde/Downloads/sml_importance .pdf");
        let filename = raw_filename.file_name().unwrap().to_str().unwrap();
        let mut file_to_w = File::open(&raw_filename).unwrap();
        let buffersize: usize = 512;

        let written = codex.add_file(file_to_w, buffersize, filename);

        assert!(written.is_ok())
    }
}
