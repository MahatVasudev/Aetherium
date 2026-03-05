fn is_delimiter(b: &u8) -> bool {
    DELIMITER.contains(b)
}

pub fn split_complete_buffer(buff: &[u8]) -> (Vec<u8>, Vec<u8>) {
    for i in (0..buff.len()).rev() {
        if is_delimiter(&buff[i]) {
            return (buff[..=i].into(), buff[(i + 1)..].into());
        }
    }

    (vec![], buff.to_vec())
}

const DELIMITER: &[u8] = &[b' ', b'/', b'\t', b'\n', b'.', b',', b';', b':', b'!'];
