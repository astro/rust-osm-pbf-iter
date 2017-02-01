use std::convert::From;
use std::str::from_utf8;
use std::iter::*;
use std::fmt;
use std::default::Default;
use std::ops::Add;

use protobuf_iter::*;


#[derive(Clone)]
pub struct DeltaEncodedIter<'a, P: Packed<'a>, T: Clone + Add<T, Output=T> + From<<P as Packed<'a>>::Item> + Default> {
    inner: PackedIter<'a, P, T>,
    last: T,
}

impl<'a, P: Packed<'a>, T: Clone + Add<T, Output=T> + From<<P as Packed<'a>>::Item> + Default> DeltaEncodedIter<'a, P, T> {
    pub fn new(value: ParseValue<'a>) -> Self {
        DeltaEncodedIter {
            inner: From::from(value),
            last: Default::default(),  // 0
        }
    }
}

impl<'a, P: Packed<'a>, T: Clone + Add<T, Output=T> + From<<P as Packed<'a>>::Item> + Default> Iterator for DeltaEncodedIter<'a, P, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
            .map(|value| {
                let current = self.last.clone() + value;
                self.last = current.clone();
                current
            })
    }
}

impl<'a, P: Clone + Packed<'a>, T: fmt::Debug + Clone + Add<T, Output=T> + From<<P as Packed<'a>>::Item> + Default> fmt::Debug for DeltaEncodedIter<'a, P, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.clone().collect::<Vec<T>>().fmt(f)
    }
}
