use crate::{delta::DeltaEncodedIter, info::Info, primitive_block::PrimitiveBlock};
use protobuf_iter::*;

pub struct DenseInfoParser<'a> {
    primitive_block: &'a PrimitiveBlock<'a>,
    versions: PackedIter<'a, PackedVarint, u32>,
    timestamps: DeltaEncodedIter<'a, PackedVarint, i64>,
    changesets: DeltaEncodedIter<'a, PackedVarint, i64>,
    uids: DeltaEncodedIter<'a, PackedVarint, i32>,
    user_sids: DeltaEncodedIter<'a, PackedVarint, i32>,
    visibles: Option<PackedIter<'a, PackedVarint, u32>>,
}

macro_rules! some {
    ($e: expr) => {
        match $e {
            Some(x) => x,
            None => return None,
        }
    };
}

impl<'a> DenseInfoParser<'a> {
    pub fn new(primitive_block: &'a PrimitiveBlock<'a>, data: &'a [u8]) -> Option<Self> {
        let iter = MessageIter::new(data);

        Some(DenseInfoParser {
            primitive_block,
            versions: some!(iter.clone().tag::<ParseValue<'a>>(1).nth(0)).packed_varints(),
            timestamps: DeltaEncodedIter::new(some!(iter.clone().tag::<ParseValue<'a>>(2).nth(0))),
            changesets: DeltaEncodedIter::new(some!(iter.clone().tag::<ParseValue<'a>>(3).nth(0))),
            uids: DeltaEncodedIter::new(some!(iter.clone().tag::<ParseValue<'a>>(4).nth(0))),
            user_sids: DeltaEncodedIter::new(some!(iter.clone().tag::<ParseValue<'a>>(5).nth(0))),
            visibles: iter
                .clone()
                .tag::<ParseValue<'a>>(6)
                .nth(0)
                .map(|value| value.packed_varints()),
        })
    }
}

impl<'a> Iterator for DenseInfoParser<'a> {
    type Item = Info<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Info {
            version: self.versions.next(),
            timestamp: self
                .timestamps
                .next()
                .map(|timestamp| self.primitive_block.convert_date(timestamp as u64)),
            changeset: self.changesets.next().map(|changesets| changesets as u64),
            uid: self.uids.next().map(|uid| uid as u32),
            user: self
                .user_sids
                .next()
                .map(|user_sid| self.primitive_block.stringtable[user_sid as usize]),
            visible: self
                .visibles
                .as_mut()
                .and_then(|visibles| visibles.next())
                .map(|visible| visible != 0),
        })
    }
}
