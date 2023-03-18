//! Connects a BusConnectible to the Bus

use super::BusConnectible;
use std::{
    fmt::{Display, Formatter, Result},
    ops::Range,
};

/// BusDevice performs address translation for BusConnectibles.
/// It is an implementation detail of Bus.connect()
#[derive(Debug)]
pub struct BusDevice {
    pub name: String,
    pub range: Range<u16>,
    device: Box<dyn BusConnectible>,
}

impl BusDevice {
    pub fn new(name: &str, range: Range<u16>, device: Box<dyn BusConnectible>) -> Self {
        BusDevice {
            name: name.to_string(),
            range,
            device,
        }
    }
    fn translate_address(&self, addr: u16) -> Option<u16> {
        let addr = addr.wrapping_sub(self.range.start);
        if addr < self.range.end {
            Some(addr)
        } else {
            None
        }
    }
}

impl BusConnectible for BusDevice {
    fn read_at(&self, addr: u16) -> Option<u8> {
        self.device.read_at(self.translate_address(addr)?)
    }
    fn write_to(&mut self, addr: u16, data: u8) {
        if let Some(addr) = self.translate_address(addr) {
            self.device.write_to(addr, data);
        }
    }
    fn get_mut(&mut self, addr: u16) -> Option<&mut u8> {
        return self.device.get_mut(addr);
    }
}

impl Display for BusDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "{} [{:04x?}]:\n{}", self.name, self.range, self.device)
    }
}
