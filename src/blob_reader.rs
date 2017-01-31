use std::str::from_utf8;
use std::io::Read;
use byteorder::{ByteOrder, BigEndian};

use protobuf_iter::*;
use blob::Blob;


pub struct BlobReader<R> {
    read: R
}

impl<R: Read> BlobReader<R> {
    pub fn new(r: R) -> Self {
        BlobReader {
            read: r
        }
    }
}

impl<R: Read> Iterator for BlobReader<R> {
    type Item = Blob;

    fn next(&mut self) -> Option<Self::Item> {
        let mut len_buf = [0; 4];
        match self.read.read(&mut len_buf) {
            Ok(4) => {
                let len = BigEndian::read_u32(&len_buf) as usize;
                let mut header_buf = Vec::with_capacity(len);
                unsafe { header_buf.set_len(len); }
                match self.read.read_exact(&mut header_buf) {
                    Ok(()) => (),
                    _ => return None
                };

                let blob_header = match parse_blob_header(&header_buf) {
                    Some(blob_header) => blob_header,
                    None => return None
                };
                let datasize = blob_header.datasize as usize;
                let mut blob_buf = Vec::with_capacity(datasize);
                unsafe { blob_buf.set_len(datasize); }
                match self.read.read_exact(&mut blob_buf) {
                    Ok(()) => (),
                    _ => return None
                };

                if ! blob_header.is_osm_data {
                    // retry next
                    self.next()
                } else {
                    match parse_blob(&blob_buf) {
                        Some(blob) =>
                            Some(blob),
                        None =>
                            // retry next
                            self.next()
                    }
                }
            },
            _ => None
        }
    }
}

struct BlobHeader {
    is_osm_data: bool,
    datasize: u32
}

fn parse_blob_header(data: &[u8]) -> Option<BlobHeader> {
    let mut blob_header = BlobHeader {
        is_osm_data: false,
        datasize: 0
    };
    for m in MessageIter::new(&data) {
        match m.tag {
            // type
            1 => {
                let value = m.value.get_data();
                if value == b"OSMData" {
                    blob_header.is_osm_data = true;
                } else if value != b"OSMHeader" {
                    println!("Encountered something other than OSM data: {}!",
                             from_utf8(value).unwrap());
                    // Immediately terminate Iterator
                    return None
                }
            },
            // datasize
            3 => {
                blob_header.datasize = From::from(m.value);
            },
            _ => ()
        }
    }
    Some(blob_header)
}

fn parse_blob(data: &[u8]) -> Option<Blob> {
    for m in MessageIter::new(&data) {
        match m.tag {
            // raw
            1 => {
                return Some(Blob::Raw(Vec::from(m.value.get_data())))
            },
            3 => {
                return Some(Blob::Zlib(Vec::from(m.value.get_data())))
            },
            _ => ()
        }
    }

    None
}

