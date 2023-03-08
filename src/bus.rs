//! The Bus connects the CPU to Memory
mod bus_device;

use crate::dump::{BinDumpable, Dumpable};
use bus_device::BusDevice;
use std::{
    fmt::{Debug, Display, Formatter, Result},
    ops::Range,
};

/// Creates a new bus, instantiating BusConnectable devices
/// # Examples
/// ```rust
/// # use rumpulator::prelude::*;
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

/// BusConnectable objects can be connected to a bus with `Bus::connect()`
///
/// The bus performs address translation, so your object will receive
/// reads and writes relative to offset 0
pub trait BusConnectible: Debug + Display {
    fn read_at(&self, addr: u16) -> Option<u8>;
    fn write_to(&mut self, addr: u16, data: u8);
}

// Traits Read and Write are here purely to make implementing other things more bearable
/// Do whatever `Read` means to you
pub trait Read<T> {
    /// Read a T from address `addr`
    fn read(&self, addr: u16) -> T;
}

/// Write "some data" to the Bus
pub trait Write<T> {
    /// Write a T to address `addr`
    fn write(&mut self, addr: u16, data: T);
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
    pub fn get_region_by_name(&self, name: &str) -> Option<Range<u16>> {
        for item in &self.devices {
            if item.name == name {
                return Some(item.range.clone());
            }
        }
        None
    }
}

/// lmao
impl BusConnectible for Bus {
    fn read_at(&self, addr: u16) -> Option<u8> {
        Some(self.read(addr))
    }
    fn write_to(&mut self, addr: u16, data: u8) {
        self.write(addr, data)
    }
}

impl Read<u8> for Bus {
    fn read(&self, addr: u16) -> u8 {
        let mut result: u8 = 0;
        for item in &self.devices {
            result |= item.read_at(addr).unwrap_or(0)
        }
        result
    }
}

impl Read<u16> for Bus {
    fn read(&self, addr: u16) -> u16 {
        let mut result = 0;
        for item in &self.devices {
            result |= (item.read_at(addr).unwrap_or(0) as u16) << 8;
            result |= item.read_at(addr.wrapping_add(1)).unwrap_or(0) as u16;
        }
        result
    }
}

impl Write<u8> for Bus {
    fn write(&mut self, addr: u16, data: u8) {
        for item in &mut self.devices {
            item.write_to(addr, data)
        }
    }
}

impl Write<u16> for Bus {
    fn write(&mut self, addr: u16, data: u16) {
        for item in &mut self.devices {
            item.write_to(addr, (data >> 8) as u8);
            item.write_to(addr.wrapping_add(1), data as u8);
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
        for index in range {
            let byte: u8 = self.read(index as u16);
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
