use std::{fs::File, io::Read, path::Path};

use crate::{
    storage::{self, error::StorageError, storage_types::FileInSystem},
    storage_assert,
    tfidf::utils,
};

pub struct ChunkReader {
    file: File,
    buffer: Vec<u8>,
    carry: Vec<u8>,
    done: bool,
}

impl ChunkReader {
    pub fn open<P: AsRef<Path>>(filename: P, chunk_size: usize) -> Result<Self, StorageError> {
        Ok(Self {
            file: File::open(filename)?,
            buffer: vec![0; chunk_size],
            carry: Vec::new(),
            done: false,
        })
    }
}

impl Iterator for ChunkReader {
    type Item = Result<String, StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            if !self.carry.is_empty() {
                let chunk = std::mem::take(&mut self.carry);
                return Some(
                    std::str::from_utf8(&chunk)
                        .map(|r| r.to_owned())
                        .map_err(|_| StorageError::Corrupt("invalid utf-8 in carry".into())),
                );
            }

            return None;
        }

        loop {
            let n = match self.file.read(&mut self.buffer) {
                Ok(val) => val,
                Err(e) => return Some(Err(e.into())),
            };

            if n == 0 {
                self.done = true;

                if !self.carry.is_empty() {
                    let chunk = std::mem::take(&mut self.carry);
                    return Some(
                        std::str::from_utf8(&chunk)
                            .map(|r| r.to_owned())
                            .map_err(|_| StorageError::Corrupt("invalid utf-8 in carry".into())),
                    );
                }

                return None;
            }

            self.carry.extend_from_slice(&self.buffer[..n]);

            let (complete_buff, carry_buff) = utils::split_complete_buffer(&self.carry);

            self.carry = carry_buff;

            if !complete_buff.is_empty() {
                return Some(
                    std::str::from_utf8(&complete_buff)
                        .map(|r| r.to_owned())
                        .map_err(|_| StorageError::Corrupt("invalid utf-8 in carry".into())),
                );
            }
        }
    }
}
