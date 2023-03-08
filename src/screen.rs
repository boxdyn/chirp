//! Stores and displays the Chip-8's screen memory

use crate::{bus::BusConnectible, dump::Dumpable, mem::Mem};
use std::{
    fmt::{Display, Formatter, Result},
    ops::Range,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Screen {
    mem: Mem,
    width: usize,
    height: usize,
}

impl Screen {
    pub fn new(width: usize, height: usize) -> Screen {
        Screen {
            mem: Mem::new(width * height / 8),
            width,
            height,
        }
    }
}

impl BusConnectible for Screen {
    fn read_at(&self, addr: u16) -> Option<u8> {
        self.mem.read_at(addr)
    }

    fn write_to(&mut self, addr: u16, data: u8) {
        self.mem.write_to(addr, data)
    }
}

impl Dumpable for Screen {
    fn dump(&self, range: Range<usize>) {
        self.mem.dump(range)
    }
}

impl Display for Screen {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.mem.window(0..self.width * self.height / 8))
    }
}
