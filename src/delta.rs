use std::convert::Into;
use std::default::Default;
use std::fmt;
use std::iter::*;
use std::ops::Add;

use protobuf_iter::*;

#[derive(Clone)]
pub struct DeltaEncodedIter<
    'a,
    P: Packed<'a>,
    T: Clone + Add<T, Output = T> + From<<P as Packed<'a>>::Item> + Default,
> {
    inner: PackedIter<'a, P, T>,
    last: T,
}

impl<'a, P: Packed<'a>, T: Clone + Add<T, Output = T> + From<<P as Packed<'a>>::Item> + Default>
    DeltaEncodedIter<'a, P, T>
{
    pub fn new(value: ParseValue<'a>) -> Self {
        DeltaEncodedIter {
            inner: value.into(),
            last: Default::default(), // 0
        }
    }
}

impl<'a, P: Packed<'a>, T: Clone + Add<T, Output = T> + From<<P as Packed<'a>>::Item> + Default>
    Iterator for DeltaEncodedIter<'a, P, T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|value| {
            let current = self.last.clone() + value;
            self.last = current.clone();
            current
        })
    }
}

impl<
    'a,
    P: Clone + Packed<'a>,
    T: fmt::Debug + Clone + Add<T, Output = T> + From<<P as Packed<'a>>::Item> + Default,
> fmt::Debug for DeltaEncodedIter<'a, P, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.clone().collect::<Vec<T>>().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::DeltaEncodedIter;
    use protobuf_iter::{PackedVarint, ParseValue};

    #[test]
    fn test_normal() {
        let mut iter: DeltaEncodedIter<PackedVarint, i64> =
            DeltaEncodedIter::new(ParseValue::LengthDelimited(&[0, 1, 6, 3]));
        assert_eq!(format!("{:?}", iter), "[0, -1, 2, 0]");
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(-1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_empty() {
        let mut iter: DeltaEncodedIter<PackedVarint, i64> =
            DeltaEncodedIter::new(ParseValue::LengthDelimited(&[]));
        assert_eq!(format!("{:?}", iter), "[]");
        assert_eq!(iter.next(), None);
    }
}
