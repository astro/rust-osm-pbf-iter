use std::hash::{Hash, Hasher};
use protobuf_iter::*;

use delta::DeltaEncodedIter;
use super::primitive_block::PrimitiveBlock;
use super::info::Info;
use super::tags::TagsIter;

#[derive(Debug, Clone)]
pub struct Way<'a> {
    pub id: u64,
    pub info: Option<Info<'a>>,
    tags_iter: TagsIter<'a>,
    refs_iter: DeltaEncodedIter<'a, PackedVarint, i64>,
}

impl<'a> Way<'a> {
    pub fn parse(primitive_block: &'a PrimitiveBlock<'a>, data: &'a [u8]) -> Self {
        let mut way = Way {
            id: 0,
            info: None,
            tags_iter: TagsIter::new(&primitive_block.stringtable),
            refs_iter: DeltaEncodedIter::new(ParseValue::LengthDelimited(&[])),
        };

        let iter = MessageIter::new(data);
        for m in iter.clone() {
            match m.tag {
                1 =>
                    way.id = Into::into(m.value),
                2 =>
                    way.tags_iter.set_keys(*m.value),
                3 =>
                    way.tags_iter.set_values(*m.value),
                4 =>
                    way.info = Some(Info::parse(&primitive_block.stringtable, *m.value)),
                8 =>
                    way.refs_iter = DeltaEncodedIter::new(m.value),
                _ => ()
            }
        }

        way
    }

    pub fn tags(&self) -> TagsIter<'a> {
        self.tags_iter.clone()
    }

    pub fn refs(&self) -> DeltaEncodedIter<'a, PackedVarint, i64> {
        self.refs_iter.clone()
    }
}

impl<'a> Hash for Way<'a> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state)
    }
}

impl<'a> Eq for Way<'a> {}

impl<'a> PartialEq for Way<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
