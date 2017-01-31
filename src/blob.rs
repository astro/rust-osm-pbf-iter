use std::io::{Cursor, Read};
use flate2::read::ZlibDecoder;


pub enum Blob {
    Raw(Vec<u8>),
    Zlib(Vec<u8>)
}

impl Blob {
    pub fn into_data(self) -> Vec<u8> {
        match self {
            Blob::Raw(data) => data,
            Blob::Zlib(compressed) => {
                // TODO: Vec::with_capacity() from raw_size
                let mut decompressed = Vec::new();
                ZlibDecoder::new(Cursor::new(compressed))
                    .read_to_end(&mut decompressed)
                    .unwrap();
                decompressed
            }
        }
    }
}
