use std::convert::From;
use std::str::from_utf8;
use std::iter::*;
use std::fmt;

use protobuf_iter::*;

use parse_dense::*;
use info::*;


const NANO: f64 = 1.0e-9;

#[derive(Clone)]
pub struct PrimitiveBlock<'a> {
    stringtable: Vec<&'a str>,
    iter: MessageIter<'a>,
    granularity: f64,
    lat_offset: f64,
    lon_offset: f64,
    date_granularity: f64,
}

#[derive(Debug)]
pub enum Primitive<'a> {
    Node(Node<'a>),
    Way(Way<'a>),
    Relation(Relation<'a>)
}

#[derive(Debug)]
pub struct Node<'a> {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    pub info: Option<InfoParser<'a>>,
    pub tags: NodeTags,
}

#[derive(Debug)]
pub struct Way<'a> {
    pub id: u64,
    pub info: Option<InfoParser<'a>>,
    tags_iter: TagsIter<'a>,
    refs_iter: WayRefsIter<'a>,
}

#[derive(Debug)]
pub struct Relation<'a> {
    pub id: u64,
    pub info: Option<InfoParser<'a>>,
    tags_iter: TagsIter<'a>,
}

#[derive(Clone)]
pub struct TagsIter<'a> {
    keys: PackedIter<'a, PackedVarint, u32>,
    vals: PackedIter<'a, PackedVarint, u32>,
    stringtable: &'a [&'a str],
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

impl<'a> Node<'a> {
    fn parse(primitive_block: &'a PrimitiveBlock<'a>, data: &'a [u8]) -> Self {
        let mut node = Node {
            id: 0,
            lat: 0.0,
            lon: 0.0,
            info: None,
            tags: NodeTags { }
        };

        let iter = MessageIter::new(data);
        for m in iter.clone() {
            match m.tag {
                1 =>
                    node.id = Into::<i64>::into(m.value) as u64,
                4 =>
                    node.info = Some(InfoParser::new(*m.value)),
                8 =>
                    node.lat = primitive_block.convert_lat(
                        Into::<i64>::into(m.value) as f64
                    ),
                9 =>
                    node.lon = primitive_block.convert_lon(
                        Into::<i64>::into(m.value) as f64
                    ),
                _ => ()
            }
        }

        node
    }
}

impl<'a> Way<'a> {
    fn parse(primitive_block: &'a PrimitiveBlock<'a>, data: &'a [u8]) -> Self {
        let mut way = Way {
            id: 0,
            info: None,
            tags_iter: TagsIter {
                keys: PackedIter::new(&[]),
                vals: PackedIter::new(&[]),
                stringtable: &primitive_block.stringtable,
            },
            refs_iter: WayRefsIter::new(ParseValue::LengthDelimited(&[])),
        };

        let iter = MessageIter::new(data);
        for m in iter.clone() {
            match m.tag {
                1 =>
                    way.id = Into::into(m.value),
                2 =>
                    way.tags_iter.keys = PackedIter::new(*m.value),
                3 =>
                    way.tags_iter.vals = PackedIter::new(*m.value),
                4 =>
                    way.info = Some(InfoParser::new(*m.value)),
                8 =>
                    way.refs_iter = WayRefsIter::new(m.value),
                _ => ()
            }
        }

        way
    }

    pub fn tags(&self) -> TagsIter<'a> {
        self.tags_iter.clone()
    }

    pub fn refs(&self) -> WayRefsIter<'a> {
        self.refs_iter.clone()
    }
}

#[derive(Clone)]
pub struct WayRefsIter<'a> {
    values: PackedIter<'a, PackedVarint, i64>,
    last: u64,
}

impl<'a> WayRefsIter<'a> {
    pub fn new(value: ParseValue<'a>) -> Self {
        WayRefsIter {
            values: From::from(value),
            last: 0,
        }
    }
}

impl<'a> Iterator for WayRefsIter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.next()
            .map(|value| {
                let current = (self.last as i64 + value) as u64;
                self.last = current;
                current
            })
    }
}

impl<'a> fmt::Debug for WayRefsIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "["));
        for (i, val) in self.clone().enumerate() {
            if i > 0 {
                try!(write!(f, ","));
            }
            try!(write!(f, " {:?}", val));
        }
        try!(write!(f, " }}"));
        Ok(())
    }
}

impl<'a> Relation<'a> {
    fn parse(primitive_block: &'a PrimitiveBlock<'a>, data: &'a [u8]) -> Self {
        let iter = MessageIter::new(data);

        let mut relation = Relation {
            id: 0,
            info: None,
            tags_iter: TagsIter {
                keys: PackedIter::new(&[]),
                vals: PackedIter::new(&[]),
                stringtable: &primitive_block.stringtable,
            },
        };

        for m in iter.clone() {
            match m.tag {
                1 =>
                    relation.id = Into::into(m.value),
                2 =>
                    relation.tags_iter.keys = PackedIter::new(*m.value),
                3 =>
                    relation.tags_iter.vals = PackedIter::new(*m.value),
                4 =>
                    relation.info = Some(InfoParser::new(*m.value)),
                _ => ()
            }
        }

        relation
    }

    pub fn tags(&self) -> TagsIter<'a> {
        self.tags_iter.clone()
    }
}

impl<'a> Iterator for TagsIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.keys.next(), self.vals.next()) {
            (Some(key_index), Some(val_index)) => {
                let key_index = key_index as usize;
                let val_index = val_index as usize;
                if key_index < self.stringtable.len() &&
                    val_index < self.stringtable.len() {
                        let key = self.stringtable[key_index];
                        let val = self.stringtable[val_index];
                        Some((key, val))
                    } else {
                        None
                    }
            },
            _ => None
        }
    }
}

impl<'a> fmt::Debug for TagsIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{{"));
        for (i, (key, val)) in self.clone().enumerate() {
            if i > 0 {
                try!(write!(f, ","));
            }
            try!(write!(f, " {}={:?}", key, val));
        }
        try!(write!(f, " }}"));
        Ok(())
    }
}

#[derive(Debug)]
pub struct NodeTags {
}
