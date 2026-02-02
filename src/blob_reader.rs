use std::io::Read;
use std::str::from_utf8;

use crate::blob::Blob;
use protobuf_iter::*;

pub struct BlobReader<R> {
    read: R,
}

impl<R: Read> BlobReader<R> {
    pub fn new(r: R) -> Self {
        BlobReader { read: r }
    }

    pub fn into_inner(self) -> R {
        self.read
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.read
    }

    pub fn read_blob(read: &mut R) -> Option<Blob> {
        let mut len_buf = [0; 4];
        match read.read(&mut len_buf) {
            Ok(4) => {
                let len = u32::from_be_bytes(len_buf) as usize;
                let header_buf = Self::read_exact(read, len).ok()?;
                let blob_header = parse_blob_header(&header_buf)?;
                let datasize = blob_header.datasize as usize;
                let blob_buf = Self::read_exact(read, datasize).ok()?;
                if !blob_header.is_osm_data {
                    // retry next
                    Self::read_blob(read)
                } else {
                    match parse_blob(&blob_buf) {
                        Some(blob) => Some(blob),
                        None =>
                        // retry next
                        {
                            Self::read_blob(read)
                        }
                    }
                }
            }
            _ => None,
        }
    }

    fn read_exact(reader: &mut R, len: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(len);
        // SAFETY: We read exactly `len` bytes.
        unsafe {
            let uninit_buf = std::slice::from_raw_parts_mut(buf.as_mut_ptr(), len);
            reader.read_exact(uninit_buf)?;
            buf.set_len(len);
        };
        Ok(buf)
    }
}

impl<R: Read> Iterator for BlobReader<R> {
    type Item = Blob;

    fn next(&mut self) -> Option<Self::Item> {
        Self::read_blob(&mut self.read)
    }
}

struct BlobHeader {
    is_osm_data: bool,
    datasize: u32,
}

fn parse_blob_header(data: &[u8]) -> Option<BlobHeader> {
    let mut blob_header = BlobHeader {
        is_osm_data: false,
        datasize: 0,
    };
    for m in MessageIter::new(&data) {
        match m.tag {
            // type
            1 => {
                let value = m.value.get_data();
                if value == b"OSMData" {
                    blob_header.is_osm_data = true;
                } else if value != b"OSMHeader" {
                    println!(
                        "Encountered something other than OSM data: {}!",
                        from_utf8(value).unwrap()
                    );
                    // Immediately terminate Iterator
                    return None;
                }
            }
            // datasize
            3 => {
                blob_header.datasize = From::from(m.value);
            }
            _ => (),
        }
    }
    Some(blob_header)
}

fn parse_blob(data: &[u8]) -> Option<Blob> {
    for m in MessageIter::new(&data) {
        match m.tag {
            // raw
            1 => return Some(Blob::Raw(Vec::from(m.value.get_data()))),
            3 => return Some(Blob::Zlib(Vec::from(m.value.get_data()))),
            _ => (),
        }
    }

    None
}
