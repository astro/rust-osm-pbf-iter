extern crate osm_pbf_iter;

use std::env::args;
use std::fs::File;

use osm_pbf_iter::*;

fn main() {
    for arg in args().skip(1) {
        let mut stats = [0; 3];

        println!("Open {}", arg);
        let f = File::open(&arg).unwrap();
        for blob in BlobReader::new(f) {
            let data = blob.into_data();
            println!("blob: {} KB", data.len() / 1024);
            let primitive_block = PrimitiveBlock::parse(&data);
            for primitive in primitive_block.primitives() {
                match primitive {
                    Primitive::Node(node) => {
                        stats[0] += 1;
                        println!("{:?}", node);
                    },
                    Primitive::Way(way) => {
                        stats[1] += 1;
                        println!("{:?}", way);
                    },
                    Primitive::Relation(relation) => {
                        stats[2] += 1;
                        println!("{:?}", relation);
                    },
                }
            }
        }

        println!("{} - {} nodes, {} ways, {} relations", arg, stats[0], stats[1], stats[2]);
    }
}
