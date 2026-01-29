use libdeflater::{DecompressionError, Decompressor};

pub enum Blob {
    Raw(Vec<u8>),
    Zlib(Vec<u8>),
}

impl Blob {
    pub fn into_data(self) -> Vec<u8> {
        match self {
            Blob::Raw(data) => data,
            Blob::Zlib(compressed) => {
                let mut decompressor = Decompressor::new();
                let mut expected_len = 4 * compressed.len();
                let mut decompressed = Vec::with_capacity(expected_len);
                loop {
                    decompressed.resize(expected_len, 0);
                    match decompressor.zlib_decompress(&compressed[..], &mut decompressed[..]) {
                        Ok(len) => {
                            assert!(len <= decompressed.len());
                            decompressed.resize(len, 0);
                            return decompressed;
                        }
                        Err(DecompressionError::InsufficientSpace) => expected_len *= 2,
                        Err(DecompressionError::BadData) => panic!("Bad zlib data"),
                    }
                }
            }
        }
    }
}
