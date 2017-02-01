use std::str::from_utf8;
use protobuf_iter::*;

use super::node::Node;
use super::way::Way;
use super::relation::Relation;
use super::dense_nodes::DenseNodesParser;

const NANO: f64 = 1.0e-9;

#[derive(Clone)]
pub struct PrimitiveBlock<'a> {
    pub stringtable: Vec<&'a str>,
    iter: MessageIter<'a>,
    pub granularity: f64,
    pub lat_offset: f64,
    pub lon_offset: f64,
    pub date_granularity: f64,
}

#[derive(Debug)]
pub enum Primitive<'a> {
    Node(Node<'a>),
    Way(Way<'a>),
    Relation(Relation<'a>),
}

impl<'a> PrimitiveBlock<'a> {
    pub fn parse(data: &'a [u8]) -> PrimitiveBlock<'a> {
        let mut result = PrimitiveBlock {
            stringtable: vec![],
            iter: MessageIter::new(&data),
            granularity: 100.0,
            lat_offset: 0.0,
            lon_offset: 0.0,
            date_granularity: 1000.0,
        };

        for m in result.iter.clone() {
            match m.tag {
                1 => result.stringtable = parse_stringtable(*m.value),
                17 => result.granularity = NANO * Into::<u32>::into(m.value) as f64,
                19 => result.lat_offset = NANO * Into::<i64>::into(m.value) as f64,
                20 => result.lon_offset = NANO * Into::<i64>::into(m.value) as f64,
                18 => result.date_granularity = Into::<u32>::into(m.value) as f64,
                _ => ()
            }
        }
        println!("[PrimitiveBlock] granularity: {:?} lat_offset: {:?} lon_offset: {:?}", result.granularity, result.lat_offset, result.lon_offset);
        println!("stringtable: {:?}", result.stringtable);

        result
    }

    pub fn convert_lat(&self, lat: f64) -> f64 {
        self.lat_offset + self.granularity * lat
    }

    pub fn convert_lon(&self, lon: f64) -> f64 {
        self.lon_offset + self.granularity * lon
    }

    // TODO: just Iterator
    pub fn primitives(&'a self) -> PrimitivesIterator<'a> {
        PrimitivesIterator {
            primitive_block: self,
            primitive_groups: self.iter.clone().tag(2),
            primitive_group: None,
            dense_nodes: None,
        }
    }
}

fn parse_stringtable<'a>(data: &'a [u8]) -> Vec<&'a str> {
    MessageIter::new(&data)
        .tag(1)
        .map(|s| from_utf8(s).expect("Malformed stringtable entry"))
        .collect()
}

pub struct PrimitivesIterator<'a> {
    primitive_block: &'a PrimitiveBlock<'a>,
    primitive_groups: ByTag<'a, ParseValue<'a>>,
    primitive_group: Option<MessageIter<'a>>,
    dense_nodes: Option<DenseNodesParser<'a>>,
}

impl<'a> Iterator for PrimitivesIterator<'a> {
    type Item = Primitive<'a>;

    fn next(&mut self) -> Option<Primitive<'a>> {
        match self.dense_nodes.take() {
            None => {
                match self.primitive_group.take() {
                    None => {
                        match self.primitive_groups.next() {
                            Some(primitive_group) => {
                                self.primitive_group = Some(From::from(primitive_group));
                                // start parsing primitive_group
                                self.next()
                            },
                            // All done
                            None => return None
                        }
                    },
                    Some(mut primitive_group) =>
                        primitive_group.next()
                        .and_then(|m| {
                            // Put back in for the next call to `next()`
                            self.primitive_group = Some(primitive_group);

                            match m.tag {
                                // node
                                1 => {
                                    let node = Node::parse(self.primitive_block, *m.value);
                                    Some(Primitive::Node(node))
                                },
                                // dense_nodes
                                2 => {
                                    self.dense_nodes = DenseNodesParser::new(self.primitive_block, *m.value);
                                    // start parsing dense_nodes:
                                    self.next()
                                },
                                // way
                                3 => {
                                    let way = Way::parse(self.primitive_block, *m.value);
                                    Some(Primitive::Way(way))
                                },
                                // relation
                                4 => {
                                    let relation = Relation::parse(self.primitive_block, *m.value);
                                    Some(Primitive::Relation(relation))
                                },
                                // skip
                                _ => self.next()
                            }
                        })
                }
            },
            Some(mut dense_nodes) => {
                match dense_nodes.next() {
                    None => {
                        // dense_nodes have been processed, no need to
                        // put back.  proceed:
                        self.next()
                    },
                    Some(dense_node) => {
                        // Put back in for the next call to `next()`
                        self.dense_nodes = Some(dense_nodes);

                        Some(Primitive::Node(dense_node))
                    }
                }
            }
        }
    }
}
