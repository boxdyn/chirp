//! Stores and displays the Chip-8's screen memory

#![allow(unused_imports)]

use crate::{bus::BusConnectible, dump::Dumpable, mem::Mem};
use std::{
    fmt::{Display, Formatter, Result},
    ops::Range,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Screen {
    pub width: usize,
    pub height: usize,
}

impl Screen {
    pub fn new(width: usize, height: usize) -> Screen {
        Screen { width, height }
    }
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            width: 64,
            height: 32,
        }
    }
}
