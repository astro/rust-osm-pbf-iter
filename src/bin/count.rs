extern crate osm_pbf_iter;

use std::env::args;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::time::Instant;

use osm_pbf_iter::*;

fn main() {
    for arg in args().skip(1) {
        let mut stats = [0; 3];

        println!("Open {}", arg);
        let f = File::open(&arg).unwrap();
        let mut reader = BlobReader::new(f);
        let start = Instant::now();

        for blob in &mut reader {
            let data = blob.into_data();
            let primitive_block = PrimitiveBlock::parse(&data);
            for primitive in primitive_block.primitives() {
                match primitive {
                    Primitive::Node(_) =>
                        stats[0] += 1,
                    Primitive::Way(_) =>
                        stats[1] += 1,
                    Primitive::Relation(_) =>
                        stats[2] += 1,
                }
            }
        }

        let stop = Instant::now();
        let duration = stop.duration_since(start);
        let duration = duration.as_secs() as f64 + (duration.subsec_nanos() as f64 / 1e9);
        let mut f = reader.to_inner();
        let pos = f.seek(SeekFrom::Current(0)).unwrap();
        let rate = pos as f64 / 1024f64 / 1024f64 / duration;
        println!("Processed {} MB in {:.2} seconds ({:.2} MB/s)",
                 pos / 1024 / 1024, duration, rate);

        println!("{} - {} nodes, {} ways, {} relations", arg, stats[0], stats[1], stats[2]);
    }
}
