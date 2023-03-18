//! Iterators for working with Busses

use super::{Bus, Read};
use std::ops::Range;

pub trait IterMut<'a> {
    type Item;
    fn next(&'a mut self) -> Option<&'a mut Self::Item>;
}

pub trait IntoIterMut<'a> {
    type Item;

    type IntoIter;

    fn into_iter(self) -> Self::IntoIter;
}

#[derive(Debug)]
pub struct BusIterator<'a> {
    range: Range<u16>,
    addr: u16,
    bus: &'a Bus,
}

impl<'a> BusIterator<'a> {
    /// Creates a new BusIterator with a specified range
    pub fn new(range: Range<u16>, bus: &'a Bus) -> BusIterator<'a> {
        BusIterator {
            addr: range.start,
            range,
            bus,
        }
    }
    pub fn range(mut self, range: Range<u16>) -> Self {
        self.range = range;
        self
    }
}

impl<'a> Iterator for BusIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let mut res = None;
        if self.range.contains(&self.addr) {
            res = Some(self.bus.read(self.addr));
            self.addr += 1;
        }
        res
    }
}
