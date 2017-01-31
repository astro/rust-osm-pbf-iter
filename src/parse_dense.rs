use std::convert::From;
use std::iter::FlatMap;

use protobuf_iter::*;
use parse::*;

pub struct DenseNodesParser<'a> {
    primitive_block: &'a PrimitiveBlock<'a>,
    id: i64,
    ids: PackedIter<'a, PackedVarint, i64>,
    lat: i64,
    lats: PackedIter<'a, PackedVarint, i64>,
    lon: i64,
    lons: PackedIter<'a, PackedVarint, i64>,
    keys_vals: PackedIter<'a, PackedVarint, i32>,
}

macro_rules! some {
    ($e: expr) => {
        match $e {
            Some(x) => x,
            None => return None,
        }
    }
}

impl<'a> DenseNodesParser<'a> {
    pub fn new(primitive_block: &'a PrimitiveBlock<'a>, data: &'a [u8]) -> Option<Self> {
        let iter = MessageIter::new(data);

        Some(DenseNodesParser {
            primitive_block: primitive_block,
            id: 0,
            ids: some!(iter.clone()
                       .tag::<ParseValue<'a>>(1)
                       .nth(0)
            ).packed_varints(),
            lat: 0,
            lats: some!(iter.clone()
                        .tag::<ParseValue<'a>>(8)
                        .nth(0)
            ).packed_varints(),
            lon: 0,
            lons: some!(iter.clone()
                        .tag::<ParseValue<'a>>(9)
                        .nth(0)
            ).packed_varints(),
            keys_vals: some!(iter.clone()
                             .tag::<ParseValue<'a>>(10)
                             .nth(0)
            ).packed_varints(),
        })
    }
}

impl<'a> Iterator for DenseNodesParser<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ids.next()
            .and_then(|id| {
                self.id += id;

                self.lats.next()
                    .and_then(|lat| {
                        self.lat += lat;

                        self.lons.next()
                            .map(|lon| {
                                self.lon += lon;

                                Node {
                                    id: self.id as u64,
                                    lat: self.primitive_block.convert_lat(self.lat as f64),
                                    lon: self.primitive_block.convert_lat(self.lon as f64),
                                    info: None,
                                    tags: NodeTags { }
                                }
                            })
                    })
            })
    }
}
