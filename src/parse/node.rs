use protobuf_iter::*;

use super::primitive_block::PrimitiveBlock;
use super::info::InfoParser;
use super::tags::TagsIter;

#[derive(Debug)]
pub struct Node<'a> {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    pub info: Option<InfoParser<'a>>,
    tags_iter: TagsIter<'a>,
}

impl<'a> Node<'a> {
    pub fn parse(primitive_block: &'a PrimitiveBlock<'a>, data: &'a [u8]) -> Self {
        let mut node = Node {
            id: 0,
            lat: 0.0,
            lon: 0.0,
            info: None,
            tags_iter: TagsIter::new(&primitive_block.stringtable),
        };

        let iter = MessageIter::new(data);
        for m in iter.clone() {
            match m.tag {
                1 =>
                    node.id = Into::<i64>::into(m.value) as u64,
                2 =>
                    node.tags_iter.set_keys(*m.value),
                3 =>
                    node.tags_iter.set_values(*m.value),
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

    pub fn tags(&self) -> TagsIter<'a> {
        self.tags_iter.clone()
    }
}
