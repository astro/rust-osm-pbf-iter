use protobuf_iter::*;
use delta::DeltaEncodedIter;
use delimited::DelimitedIter;
use dense_info::DenseInfoParser;
use super::*;

pub struct DenseNodesParser<'a> {
    primitive_block: &'a PrimitiveBlock<'a>,
    ids: DeltaEncodedIter<'a, PackedVarint, i64>,
    lats: DeltaEncodedIter<'a, PackedVarint, i64>,
    lons: DeltaEncodedIter<'a, PackedVarint, i64>,
    infos: Option<DenseInfoParser<'a>>,
    keys_vals: DelimitedIter<'a, PackedVarint, u32>,
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

        // println!("keys_vals: {:?}", some!(iter.clone()
        //               .tag::<ParseValue<'a>>(10)
        //               .nth(0)).packed_varints::<u32>().collect::<Vec<u32>>());
        // println!("infos: {:?}", iter.clone()
        //          .tag::<ParseValue<'a>>(5)
        //          .nth(0));

        Some(DenseNodesParser {
            primitive_block: primitive_block,
            ids: DeltaEncodedIter::new(
                some!(iter.clone()
                      .tag::<ParseValue<'a>>(1)
                      .nth(0)
                )
            ),
            lats: DeltaEncodedIter::new(
                some!(iter.clone()
                        .tag::<ParseValue<'a>>(8)
                        .nth(0)
                )
            ),
            lons: DeltaEncodedIter::new(
                some!(iter.clone()
                      .tag::<ParseValue<'a>>(9)
                      .nth(0)
                )
            ),
            infos: iter.clone()
                .tag::<ParseValue<'a>>(5)
                .nth(0)
                .and_then(|value|
                     DenseInfoParser::new(&primitive_block, *value)
                ),
            keys_vals: DelimitedIter::new(
                some!(iter.clone()
                      .tag::<ParseValue<'a>>(10)
                      .nth(0)
                )
            ),
        })
    }
}

impl<'a> Iterator for DenseNodesParser<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut keys_vals = some!(self.keys_vals.next()).into_iter();
        let mut tags = Vec::with_capacity(keys_vals.clone().count());
        loop {
            let k = match keys_vals.next() {
                Some(k) =>
                    self.primitive_block.stringtable[k as usize],
                None => break,
            };
            let v = match keys_vals.next() {
                Some(v) =>
                    self.primitive_block.stringtable[v as usize],
                None => break,
            };
            tags.push((k, v));
        }

        let info = self.infos.as_mut()
            .and_then(|infos| infos.next());

        Some(Node {
            id: some!(self.ids.next()) as u64,
            lat: self.primitive_block.convert_lat(
                some!(self.lats.next())
            ),
            lon: self.primitive_block.convert_lon(
                some!(self.lons.next())
            ),
            info: info,
            tags: tags,
        })
    }
}
