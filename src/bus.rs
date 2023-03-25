//! The Bus connects the CPU to Memory

use crate::error::Result;
use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    ops::Range,
    slice::SliceIndex,
};

/// Creates a new bus, instantiating BusConnectable devices
/// # Examples
/// ```rust
/// # use chirp::prelude::*;
/// let mut bus = bus! {
///     Stack   [0x0000..0x0800] = b"ABCDEF",
///     Program [0x0800..0x1000] = include_bytes!("bus.rs"),
/// };
/// ```
#[macro_export]
macro_rules! bus {
    ($($name:path $(:)? [$range:expr] $(= $data:expr)?) ,* $(,)?) => {
        $crate::bus::Bus::new()
        $(
            .add_region($name, $range)
            $(
                .load_region($name, $data)
            )?
        )*
    };
}

// Traits Read and Write are here purely to make implementing other things more bearable
/// Do whatever `Read` means to you
pub trait Read<T> {
    /// Read a T from address `addr`
    fn read(&self, addr: impl Into<usize>) -> T;
}

/// Write "some data" to the Bus
pub trait Write<T> {
    /// Write a T to address `addr`
    fn write(&mut self, addr: impl Into<usize>, data: T);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Region {
    Charset,
    Program,
    Screen,
    Stack,
}

impl Display for Region {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Region::Charset => "charset",
            Region::Program => "program",
            Region::Screen => "screen",
            Region::Stack => "stack",
        })
    }
}

/// Store memory in a series of named regions with ranges
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bus {
    memory: Vec<u8>,
    region: HashMap<Region, Range<usize>>,
}

impl Bus {
    /// Construct a new bus
    pub fn new() -> Self {
        Bus::default()
    }
    /// Gets the length of the bus' backing memory
    pub fn len(&self) -> usize {
        self.memory.len()
    }
    /// Returns true if the backing memory contains no elements
    pub fn is_empty(&self) -> bool {
        self.memory.is_empty()
    }
    /// Grows the Bus backing memory to at least size bytes, but does not truncate
    pub fn with_size(&mut self, size: usize) {
        if self.len() < size {
            self.memory.resize(size, 0);
        }
    }
    pub fn add_region(mut self, name: Region, range: Range<usize>) -> Self {
        self.with_size(range.end);
        self.region.insert(name, range);
        self
    }
    pub fn load_region(mut self, name: Region, data: &[u8]) -> Self {
        use std::io::Write;
        if let Some(mut region) = self.get_region_mut(name) {
            region.write(data).ok(); // TODO: THIS SUCKS
        }
        self
    }
    pub fn clear_region(&mut self, name: Region) -> &mut Self {
        if let Some(region) = self.get_region_mut(name) {
            region.fill(0)
        }
        self
    }
    /// Gets a slice of bus memory
    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.memory.get(index)
    }
    /// Gets a mutable slice of bus memory
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.memory.get_mut(index)
    }
    /// Gets a slice of a named region of memory
    pub fn get_region(&self, name: Region) -> Option<&[u8]> {
        self.get(self.region.get(&name)?.clone())
    }
    /// Gets a mutable slice to a named region of memory
    pub fn get_region_mut(&mut self, name: Region) -> Option<&mut [u8]> {
        self.get_mut(self.region.get(&name)?.clone())
    }
    pub fn print_screen(&self) -> Result<()> {
        const REGION: Region = Region::Screen;
        if let Some(screen) = self.get_region(REGION) {
            for (index, byte) in screen.iter().enumerate() {
                if index % 8 == 0 {
                    print!("|");
                }
                print!(
                    "{}",
                    format!("{byte:08b}").replace('0', "  ").replace('1', "██")
                );
                if index % 8 == 7 {
                    println!("|");
                }
            }
        } else {
            return Err(crate::error::Error::MissingRegion {
                region: REGION.to_string(),
            });
        }
        Ok(())
    }
}

impl Read<u8> for Bus {
    fn read(&self, addr: impl Into<usize>) -> u8 {
        let addr: usize = addr.into();
        *self.memory.get(addr).unwrap_or(&0xc5)
    }
}

impl Read<u16> for Bus {
    fn read(&self, addr: impl Into<usize>) -> u16 {
        let addr: usize = addr.into();
        if let Some(bytes) = self.memory.get(addr..addr + 2) {
            u16::from_be_bytes(bytes.try_into().expect("asked for 2 bytes, got != 2 bytes"))
        } else {
            0xc5c5
        }
    }
}

impl Write<u8> for Bus {
    fn write(&mut self, addr: impl Into<usize>, data: u8) {
        let addr: usize = addr.into();
        if let Some(byte) = self.get_mut(addr) {
            *byte = data;
        }
    }
}

impl Write<u16> for Bus {
    fn write(&mut self, addr: impl Into<usize>, data: u16) {
        let addr: usize = addr.into();
        if let Some(slice) = self.get_mut(addr..addr + 2) {
            data.to_be_bytes().as_mut().swap_with_slice(slice);
        }
    }
}

impl Display for Bus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use rhexdump::Rhexdump;
        let mut rhx = Rhexdump::default();
        rhx.set_bytes_per_group(2)
            .expect("2 <= MAX_BYTES_PER_GROUP (8)");
        rhx.display_duplicate_lines(false);
        for (&name, range) in &self.region {
            writeln!(
                f,
                "[{name}]\n{}\n",
                rhx.hexdump(&self.memory[range.clone()])
            )?
        }
        write!(f, "")
    }
}
