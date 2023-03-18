//! The Bus connects the CPU to Memory
mod bus_device;
mod iterator;

use crate::dump::{BinDumpable, Dumpable};
use bus_device::BusDevice;
use iterator::BusIterator;
use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter, Result},
    ops::Range,
    slice::SliceIndex,
};

/// Creates a new bus, instantiating BusConnectable devices
/// # Examples
/// ```rust
/// # use chumpulator::prelude::*;
/// let mut bus = bus! {
///     "RAM" [0x0000..0x8000] Mem::new(0x8000),
///     "ROM" [0x8000..0xFFFF] Mem::new(0x8000).w(false),
/// };
/// ```
#[macro_export]
macro_rules! bus {
    ($($name:literal $(:)? [$range:expr] $(=)? $d:expr) ,* $(,)?) => {
        $crate::bus::Bus::new()
        $(
            .connect($name, $range, Box::new($d))
        )*
    };
}

#[macro_export]
macro_rules! newbus {
    ($($name:literal $(:)? [$range:expr] $(= $data:expr)?) ,* $(,)?) => {
        $crate::bus::NewBus::new()
        $(
            .add_region($name, $range)
            $(
                .load_region($name, $data)
            )?
        )*
    };
}

/// Store memory in a series of named regions with ranges
#[derive(Debug, Default)]
pub struct NewBus {
    memory: Vec<u8>,
    region: HashMap<&'static str, Range<usize>>,
}

impl NewBus {
    /// Construct a new bus
    pub fn new() -> Self {
        NewBus::default()
    }
    /// Gets the length of the bus' backing memory
    pub fn len(&self) -> usize {
        self.memory.len()
    }
    /// Returns true if the backing memory contains no elements
    pub fn is_empty(&self) -> bool {
        self.memory.is_empty()
    }
    /// Grows the NewBus backing memory to at least size bytes, but does not truncate
    pub fn with_size(&mut self, size: usize){
        if self.len() < size {
            self.memory.resize(size, 0);
        }
    }
    pub fn add_region(mut self, name: &'static str, range: Range<usize>) -> Self {
        self.with_size(range.end);
        self.region.insert(name, range);
        self
    }
    pub fn load_region(mut self, name: &str, data: &[u8]) -> Self {
        use std::io::Write;
        if let Some(mut region) = self.get_region_mut(name) {
            dbg!(region.write(data)).ok(); // TODO: THIS SUCKS
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
    pub fn get_region(&self, name: &str) -> Option<&[u8]> {
        self.get(self.region.get(name)?.clone())
    }
    /// Gets a mutable slice to a named region of memory
    pub fn get_region_mut(&mut self, name: &str) -> Option<&mut [u8]> {
        self.get_mut(self.region.get(name)?.clone())
    }
}

impl Read<u8> for NewBus {
    fn read(&self, addr: impl Into<usize>) -> u8 {
        *self.memory.get(addr.into()).unwrap_or(&0xc5)
    }
}

impl Read<u16> for NewBus {
    fn read(&self, addr: impl Into<usize>) -> u16 {
        let addr: usize = addr.into();
        if let Some(bytes) = self.memory.get(addr..addr + 2) {
            u16::from_be_bytes(bytes.try_into().expect("asked for 2 bytes, got != 2 bytes"))
        } else {
            0xc5c5
        }
        //u16::from_le_bytes(self.memory.get([addr;2]))
    }
}

impl Write<u8> for NewBus {
    fn write(&mut self, addr: impl Into<usize>, data: u8) {
        let addr: usize = addr.into();
        if let Some(byte) = self.get_mut(addr) {
            *byte = data;
        }
    }
}

impl Write<u16> for NewBus {
    fn write(&mut self, addr: impl Into<usize>, data: u16) {
        let addr: usize = addr.into();
        if let Some(slice) = self.get_mut(addr..addr + 2) {
            slice.swap_with_slice(data.to_be_bytes().as_mut())
        }
    }
}

impl Display for NewBus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use rhexdump::Rhexdump;
        let mut rhx = Rhexdump::default();
        rhx.set_bytes_per_group(2).expect("2 <= MAX_BYTES_PER_GROUP (8)");
        rhx.display_duplicate_lines(false);
        write!(f, "{}", rhx.hexdump(&self.memory))
    }
}

/// BusConnectable objects can be connected to a bus with `Bus::connect()`
///
/// The bus performs address translation, so your object will receive
/// reads and writes relative to offset 0
pub trait BusConnectible: Debug + Display {
    fn read_at(&self, addr: u16) -> Option<u8>;
    fn write_to(&mut self, addr: u16, data: u8);
    fn get_mut(&mut self, addr: u16) -> Option<&mut u8>;
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

/// The Bus connects bus readers with bus writers.
/// The design assumes single-threaded operation.
#[derive(Debug, Default)]
pub struct Bus {
    devices: Vec<BusDevice>,
}

impl Bus {
    /// Construct a new bus
    pub fn new() -> Self {
        Bus::default()
    }
    /// Connect a BusConnectible object to the bus
    pub fn connect(
        mut self,
        name: &str,
        range: Range<u16>,
        device: Box<dyn BusConnectible>,
    ) -> Self {
        self.devices.push(BusDevice::new(name, range, device));
        self
    }
    pub fn get_region_by_name(&mut self, name: &str) -> Option<&mut BusDevice> {
        self.devices.iter_mut().find(|item| item.name == name)
    }
}

/// lmao
impl BusConnectible for Bus {
    fn read_at(&self, addr: u16) -> Option<u8> {
        let mut result: u8 = 0;
        for item in &self.devices {
            result |= item.read_at(addr).unwrap_or(0)
        }
        Some(result)
    }
    fn write_to(&mut self, addr: u16, data: u8) {
        for item in &mut self.devices {
            item.write_to(addr, data)
        }
    }

    fn get_mut(&mut self, addr: u16) -> Option<&mut u8> {
        for item in &mut self.devices {
            if let Some(mutable) = item.get_mut(addr) {
                return Some(mutable);
            }
        }
        None
    }
}

impl Read<u8> for Bus {
    fn read(&self, addr: impl Into<usize>)  -> u8 {
        let addr = addr.into() as u16;
        let mut result: u8 = 0;
        for item in &self.devices {
            result |= item.read_at(addr).unwrap_or(0)
        }
        result
    }
}

impl Read<u16> for Bus {
    fn read(&self, addr: impl Into<usize>) -> u16 {
        let addr = addr.into() as u16;
        let mut result = 0;
        result |= (self.read_at(addr).unwrap_or(0) as u16) << 8;
        result |= self.read_at(addr.wrapping_add(1)).unwrap_or(0) as u16;
        result
    }
}

impl Write<u8> for Bus {
    fn write(&mut self, addr: impl Into<usize>, data: u8) {
        let addr = addr.into() as u16;
        for item in &mut self.devices {
            item.write_to(addr, data)
        }
    }
}

impl Write<u16> for Bus {
    fn write(&mut self, addr: impl Into<usize>, data: u16) {
        let addr = addr.into() as u16;
        self.write_to(addr, (data >> 8) as u8);
        self.write_to(addr.wrapping_add(1), data as u8);
    }
}

impl Write<u32> for Bus {
    fn write(&mut self, addr: impl Into<usize>, data: u32) {
        let addr = addr.into() as u16;
        for i in 0..4 {
            self.write_to(addr.wrapping_add(i), (data >> (3 - i * 8)) as u8);
        }
    }
}

impl Display for Bus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for device in &self.devices {
            write!(f, "{device}")?;
        }
        write!(f, "")
    }
}

impl Dumpable for Bus {
    fn dump(&self, range: Range<usize>) {
        for (index, byte) in self
            .into_iter()
            .range(range.start as u16..range.end as u16) // this causes a truncation
            .enumerate()
        {
            crate::dump::as_hexdump(index, byte);
        }
    }
}

impl BinDumpable for Bus {
    fn bin_dump(&self, range: Range<usize>) {
        for index in range {
            let byte: u8 = self.read(index as u16);
            crate::dump::as_bindump(index, byte)
        }
    }
}

impl<'a> IntoIterator for &'a Bus {
    type Item = u8;

    type IntoIter = iterator::BusIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BusIterator::new(0..u16::MAX, self)
    }
}
