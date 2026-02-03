use protobuf_iter::*;
use std::fmt;

#[derive(Clone)]
pub struct TagsIter<'a> {
    keys: Option<PackedIter<'a, PackedVarint, u32>>,
    values: Option<PackedIter<'a, PackedVarint, u32>>,
    stringtable: &'a [&'a str],
}

impl<'a> TagsIter<'a> {
    pub fn new(stringtable: &'a [&'a str]) -> Self {
        TagsIter {
            keys: None,
            values: None,
            stringtable,
        }
    }

    pub fn set_keys(&mut self, buf: &'a [u8]) {
        self.keys = Some(PackedIter::new(buf));
    }

    pub fn set_values(&mut self, buf: &'a [u8]) {
        self.values = Some(PackedIter::new(buf));
    }
}

impl<'a> Iterator for TagsIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let obtain = |opt_iter: &mut Option<PackedIter<'a, PackedVarint, u32>>| {
            opt_iter.as_mut().and_then(|iter| iter.next())
        };
        match (obtain(&mut self.keys), obtain(&mut self.values)) {
            (Some(key_index), Some(val_index)) => {
                let key_index = key_index as usize;
                let val_index = val_index as usize;
                if key_index < self.stringtable.len() && val_index < self.stringtable.len() {
                    let key = self.stringtable[key_index];
                    let val = self.stringtable[val_index];
                    Some((key, val))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<'a> fmt::Debug for TagsIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for (i, (key, val)) in self.clone().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, " {}={:?}", key, val)?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::TagsIter;

    const STRINGTABLE: [&str; 4] = ["highway", "yes", "crossing", "lit"];

    #[test]
    fn test_normal() {
        let mut iter = TagsIter::new(&STRINGTABLE);
        iter.set_keys(&[0, 3]);
        iter.set_values(&[2, 1]);
        assert_eq!(
            format!("{:?}", iter),
            "{ highway=\"crossing\", lit=\"yes\" }"
        );
        assert_eq!(iter.next(), Some(("highway", "crossing")));
        assert_eq!(iter.next(), Some(("lit", "yes")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_no_keys() {
        let mut iter = TagsIter::new(&STRINGTABLE);
        iter.set_values(&[2, 1]);
        assert_eq!(format!("{:?}", iter), "{ }");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_no_values() {
        let mut iter = TagsIter::new(&STRINGTABLE);
        iter.set_keys(&[0, 3]);
        assert_eq!(format!("{:?}", iter), "{ }");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_no_keys_no_values() {
        let mut iter = TagsIter::new(&STRINGTABLE);
        assert_eq!(format!("{:?}", iter), "{ }");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_fewer_keys_than_values() {
        let mut iter = TagsIter::new(&STRINGTABLE);
        iter.set_keys(&[0]);
        iter.set_values(&[2, 1]);
        assert_eq!(format!("{:?}", iter), "{ highway=\"crossing\" }");
        assert_eq!(iter.next(), Some(("highway", "crossing")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_fewer_values_than_keys() {
        let mut iter = TagsIter::new(&STRINGTABLE);
        iter.set_keys(&[0, 3]);
        iter.set_values(&[2]);
        assert_eq!(format!("{:?}", iter), "{ highway=\"crossing\" }");
        assert_eq!(iter.next(), Some(("highway", "crossing")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_key_overflows_stringtable() {
        let mut iter = TagsIter::new(&STRINGTABLE);
        iter.set_keys(&[4, 3]);
        iter.set_values(&[2, 1]);
        assert_eq!(format!("{:?}", iter), "{ }");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_value_overflows_stringtable() {
        let mut iter = TagsIter::new(&STRINGTABLE);
        iter.set_keys(&[0, 3]);
        iter.set_values(&[2, 210]);
        assert_eq!(format!("{:?}", iter), "{ highway=\"crossing\" }");
        assert_eq!(iter.next(), Some(("highway", "crossing")));
        assert_eq!(iter.next(), None);
    }
}
