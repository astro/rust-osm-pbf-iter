use protobuf_iter::*;
use std::str::from_utf8_unchecked;

use super::dense_nodes::DenseNodesParser;
use super::node::Node;
use super::relation::Relation;
use super::way::Way;

const NANO: f64 = 1.0e-9;

#[derive(Clone)]
pub struct PrimitiveBlock<'a> {
    pub stringtable: Vec<&'a str>,
    iter: MessageIter<'a>,
    pub granularity: u32,
    pub lat_offset: i64,
    pub lon_offset: i64,
    pub date_granularity: u64,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Primitive<'a> {
    Node(Node<'a>),
    Way(Way<'a>),
    Relation(Relation<'a>),
}

impl<'a> PrimitiveBlock<'a> {
    pub fn parse(data: &'a [u8]) -> PrimitiveBlock<'a> {
        let mut result = PrimitiveBlock {
            stringtable: vec![],
            iter: MessageIter::new(data),
            granularity: 100,
            lat_offset: 0,
            lon_offset: 0,
            date_granularity: 1000,
        };

        for m in result.iter.clone() {
            match m.tag {
                1 => result.stringtable = parse_stringtable(*m.value),
                17 => result.granularity = Into::<u32>::into(m.value),
                19 => result.lat_offset = Into::<i64>::into(m.value),
                20 => result.lon_offset = Into::<i64>::into(m.value),
                18 => result.date_granularity = Into::<u64>::into(m.value),
                _ => (),
            }
        }
        // println!("[PrimitiveBlock] granularity: {:?} lat_offset: {:?} lon_offset: {:?}", result.granularity, result.lat_offset, result.lon_offset);
        // println!("stringtable: {:?}", result.stringtable);

        result
    }

    pub fn convert_lat(&self, lat: i64) -> f64 {
        NANO * (self.lat_offset + self.granularity as i64 * lat) as f64
    }

    pub fn convert_lon(&self, lon: i64) -> f64 {
        NANO * (self.lon_offset + self.granularity as i64 * lon) as f64
    }

    // should return timestamp in milliseconds since 1970
    pub fn convert_date(&self, date: u64) -> u64 {
        self.date_granularity * date
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

fn parse_stringtable(data: &[u8]) -> Vec<&str> {
    MessageIter::new(data)
        .tag(1)
        .map(|s| unsafe { from_utf8_unchecked(s) })
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
        // Try to yield a Primitive::Node from dense_nodes.
        if let Some(dense_nodes) = &mut self.dense_nodes {
            if let Some(node) = dense_nodes.next() {
                return Some(Primitive::Node(node));
            }
            self.dense_nodes = None; // iterator exhausted
        }

        // Try to yield a Primitive from the current primitive_group.
        if let Some(group) = &mut self.primitive_group {
            for m in group.by_ref() {
                match m.tag {
                    // node
                    1 => {
                        let node = Node::parse(self.primitive_block, *m.value);
                        return Some(Primitive::Node(node));
                    }

                    // dense_nodes
                    2 => {
                        self.dense_nodes = DenseNodesParser::new(self.primitive_block, *m.value);
                        return self.next(); // start parsing dense_nodes
                    }

                    // way
                    3 => {
                        let way = Way::parse(self.primitive_block, *m.value);
                        return Some(Primitive::Way(way));
                    }

                    // relation
                    4 => {
                        let relation = Relation::parse(self.primitive_block, *m.value);
                        return Some(Primitive::Relation(relation));
                    }

                    _ => continue,
                }
            }
            self.primitive_group = None; // iterator exhausted
        }

        // Try to yield something from the next primitive group.
        if let Some(group) = self.primitive_groups.next() {
            self.primitive_group = Some(MessageIter::from(group));
            return self.next(); // start parsing primitive_group
        }

        None
    }
}
