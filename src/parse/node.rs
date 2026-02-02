use protobuf_iter::*;
use std::hash::{Hash, Hasher};

use super::info::Info;
use super::primitive_block::PrimitiveBlock;
use super::tags::TagsIter;

#[derive(Debug, Clone)]
pub struct Node<'a> {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    pub info: Option<Info<'a>>,
    pub tags: Vec<(&'a str, &'a str)>,
}

impl<'a> Node<'a> {
    pub fn parse(primitive_block: &'a PrimitiveBlock<'a>, data: &'a [u8]) -> Self {
        let mut id = 0;
        let mut lat = 0.0;
        let mut lon = 0.0;
        let mut info = None;
        let mut tags_iter = TagsIter::new(&primitive_block.stringtable);

        let iter = MessageIter::new(data);
        for m in iter.clone() {
            match m.tag {
                1 => id = Into::<i64>::into(m.value) as u64,
                2 => tags_iter.set_keys(*m.value),
                3 => tags_iter.set_values(*m.value),
                4 => info = Some(Info::parse(&primitive_block.stringtable, *m.value)),
                8 => lat = primitive_block.convert_lat(Into::<i64>::into(m.value)),
                9 => lon = primitive_block.convert_lon(Into::<i64>::into(m.value)),
                _ => (),
            }
        }

        Node {
            id,
            lat,
            lon,
            info,
            tags: tags_iter.collect(),
        }
    }
}

impl<'a> Hash for Node<'a> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state)
    }
}

impl<'a> Eq for Node<'a> {}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
