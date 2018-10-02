use std::convert::From;
use std::iter::*;
use std::default::Default;
use std::ops::Add;

use protobuf_iter::*;


#[derive(Clone)]
pub struct DelimitedIter<'a, P: Packed<'a>, T: Clone + Add<T, Output=T> + From<<P as Packed<'a>>::Item> + Default> {
    inner: PackedIter<'a, P, T>,
}

impl<'a, P: Packed<'a>, T: Clone + Add<T, Output=T> + From<<P as Packed<'a>>::Item> + Default> DelimitedIter<'a, P, T> {
    pub fn new(value: ParseValue<'a>) -> Self {
        DelimitedIter {
            inner: From::from(value),
        }
    }
}

impl<'a, P: Packed<'a>, T: Clone + Add<T, Output=T> + From<<P as Packed<'a>>::Item> + Default + PartialEq> Iterator for DelimitedIter<'a, P, T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = vec![];

        for el in &mut self.inner {
            // == 0?
            if el == Default::default() {
                break
            }
            result.push(el);
        }

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = self.inner.size_hint();
        (0, Some(hint.1.unwrap_or(hint.0)))
    }
}
