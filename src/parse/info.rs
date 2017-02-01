use std::fmt;

use protobuf_iter::*;

macro_rules! protobuf_getter {
    ($name: ident, $tag: expr, $value_type: ty, $default: expr) => {
        pub fn $name(&self) -> $value_type {
            let mut result = $default;
            match self.iter.clone().tag($tag).nth(0) {
                Some(value) => result = value,
                None => (),
            }
            result
        }
    }
}


pub struct InfoParser<'a> {
    iter: MessageIter<'a>
}

impl<'a> InfoParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        InfoParser {
            iter: MessageIter::new(data)
        }
    }

    protobuf_getter!(version, 1, u32, 0);
    protobuf_getter!(timestamp, 2, u64, 0);
    protobuf_getter!(changeset, 3, u64, 0);
    protobuf_getter!(uid, 4, u32, 0);
    // TODO: user_sid
}

impl<'a> fmt::Debug for InfoParser<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v{} @{}-{} by {}", self.version(), self.timestamp(), self.changeset(), self.uid())
    }
}
