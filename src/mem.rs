//! Mem covers WOM, ROM, and RAM

use crate::{bus::BusConnectible, dump::Dumpable};
use owo_colors::{OwoColorize, Style};
use std::{
    fmt::{Display, Formatter, Result},
    ops::Range,
};

const MSIZE: usize = 0x1000;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Attr {
    pub r: bool,
    pub w: bool,
}

pub struct MemWindow<'a> {
    mem: &'a [u8],
}

impl<'a> Display for MemWindow<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Green phosphor style formatting, for taste
        let term: Style = Style::new().bold().green().on_black();
        for (index, byte) in self.mem.iter().enumerate() {
            if index % 16 == 0 {
                write!(f, "{:>03x}{} ", index.style(term), ":".style(term))?
            }
            write!(f, "{byte:02x}")?;
            write!(
                f,
                "{}",
                match index % 16 {
                    0xf => "\n",
                    0x7 => "  ",
                    _ if index % 2 == 1 => " ",
                    _ => "",
                }
            )?
        }
        write!(f, "")
    }
}

/// Represents some kind of abstract memory chip
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mem {
    mem: Vec<u8>,
    attr: Attr,
}

impl Mem {
    pub fn r(mut self, readable: bool) -> Self {
        self.attr.r = readable;
        self
    }
    pub fn w(mut self, writable: bool) -> Self {
        self.attr.w = writable;
        self
    }
    /// Returns the number of bytes in the `Mem`, also referred to as its length.
    ///
    /// # Examples
    /// ``` rust
    /// # use rumpulator::prelude::*;
    /// let mem = Mem::new(0x100);
    /// assert_eq!(mem.len(), 0x100)
    /// ```
    pub fn len(&self) -> usize {
        self.mem.len()
    }

    /// Because clippy is so kind:
    pub fn is_empty(&self) -> bool {
        self.mem.is_empty()
    }

    /// Loads data into a `Mem`
    pub fn load(mut self, addr: u16, bytes: &[u8]) -> Self {
        let addr = addr as usize;
        let end = self.mem.len().min(addr + bytes.len());
        self.mem[addr..end].copy_from_slice(&bytes[0..bytes.len()]);
        self
    }

    /// Load a character set from rumpulator/src/mem/charset.bin into this memory section
    pub fn load_charset(self, addr: u16) -> Self {
        let charset = include_bytes!("mem/charset.bin");
        self.load(addr, charset)
    }

    /// Creates a new `mem` with the specified length
    ///
    /// # Examples
    /// ```rust
    /// # use rumpulator::prelude::*;
    /// let length = 0x100;
    /// let mem = Mem::new(length);
    /// ```
    pub fn new(len: usize) -> Self {
        Mem {
            mem: vec![0; len],
            attr: Attr { r: true, w: true },
        }
    }

    /// Creates a window into the Mem which implements Display
    pub fn window(&self, range: Range<usize>) -> MemWindow {
        MemWindow {
            mem: &self.mem[range],
        }
    }
}

impl Default for Mem {
    fn default() -> Self {
        Self::new(MSIZE)
    }
}

impl Display for Mem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.window(0..self.len()))
    }
}

impl Dumpable for Mem {
    fn dump(&self, range: Range<usize>) {
        print!("Mem {range:2x?}: ");
        print!("{}", self.window(range));
    }
}

impl BusConnectible for Mem {
    fn read_at(&self, addr: u16) -> Option<u8> {
        if !self.attr.r {
            return None;
        }
        self.mem.get(addr as usize).copied()
    }
    fn write_to(&mut self, addr: u16, data: u8) {
        if self.attr.w && let Some(value) = self.mem.get_mut(addr as usize) {
                *value = data
        }
    }
}
