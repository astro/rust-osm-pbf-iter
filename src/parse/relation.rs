use std::fmt;
use std::hash::{Hash, Hasher};
use protobuf_iter::*;

use delta::DeltaEncodedIter;
use super::primitive_block::PrimitiveBlock;
use super::info::Info;
use super::tags::TagsIter;

#[derive(Debug, Clone)]
pub struct Relation<'a> {
    pub id: u64,
    pub info: Option<Info<'a>>,
    tags_iter: TagsIter<'a>,
    rels_iter: RelationMembersIter<'a>,
}

impl<'a> Hash for Relation<'a> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state)
    }
}

impl<'a> Eq for Relation<'a> {
}
impl<'a> PartialEq for Relation<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum RelationMemberType {
    Node,
    Way,
    Relation,
}

#[derive(Clone)]
pub struct RelationMembersIter<'a> {
    roles_sid: PackedIter<'a, PackedVarint, i32>,
    memids: DeltaEncodedIter<'a, PackedVarint, i64>,
    memid: i64,
    types: PackedIter<'a, PackedVarint, u32>,
    stringtable: &'a [&'a str],
}

impl<'a> Iterator for RelationMembersIter<'a> {
    type Item = (&'a str, u64, RelationMemberType);

    fn next(&mut self) -> Option<Self::Item> {
        let role_sid = self.roles_sid.next()? as usize;
        let role = if role_sid < self.stringtable.len() {
            self.stringtable[role_sid as usize]
        } else {
            return None
        };

        let memid_delta = self.memids.next()?;
        self.memid += memid_delta;

        let memtype = match self.types.next() {
            Some(0) => RelationMemberType::Node,
            Some(1) => RelationMemberType::Way,
            Some(2) => RelationMemberType::Relation,
            _ => return None,
        };

        Some((role, self.memid as u64, memtype))
    }
}

impl<'a> fmt::Debug for RelationMembersIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{{"));
        for (i, (role, id, reltype)) in self.clone().enumerate() {
            if i > 0 {
                try!(write!(f, ","));
            }
            try!(write!(f, " {:?} {} {:?}", reltype, id, role));
        }
        try!(write!(f, " }}"));
        Ok(())
    }
}




impl<'a> Relation<'a> {
    pub fn parse(primitive_block: &'a PrimitiveBlock<'a>, data: &'a [u8]) -> Self {
        let iter = MessageIter::new(data);

        let mut relation = Relation {
            id: 0,
            info: None,
            tags_iter: TagsIter::new(&primitive_block.stringtable),
            rels_iter: RelationMembersIter {
                roles_sid: PackedIter::new(&[]),
                memids: DeltaEncodedIter::new(ParseValue::LengthDelimited(&[])),
                memid: 0,
                types: PackedIter::new(&[]),
                stringtable: &primitive_block.stringtable,
            },
        };

        for m in iter.clone() {
            match m.tag {
                1 =>
                    relation.id = Into::into(m.value),
                2 =>
                    relation.tags_iter.set_keys(*m.value),
                3 =>
                    relation.tags_iter.set_values(*m.value),
                4 =>
                    relation.info = Some(Info::parse(&primitive_block.stringtable, *m.value)),
                8 =>
                    relation.rels_iter.roles_sid = PackedIter::new(*m.value),
                9 =>
                    relation.rels_iter.memids = DeltaEncodedIter::new(m.value),
                10 =>
                    relation.rels_iter.types = PackedIter::new(*m.value),
                _ => ()
            }
        }

        relation
    }

    pub fn tags(&self) -> TagsIter<'a> {
        self.tags_iter.clone()
    }

    pub fn members(&self) -> RelationMembersIter<'a> {
        self.rels_iter.clone()
    }
}
