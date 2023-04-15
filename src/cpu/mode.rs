//! Selects the memory behavior of the [super::CPU]

use crate::error::Error;
use std::str::FromStr;

/// Selects the memory behavior of the interpreter
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    /// VIP emulation mode
    #[default]
    Chip8,
    /// Chip-48 emulation mode
    SChip,
    /// XO-Chip emulation mode
    XOChip,
}

impl FromStr for Mode {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "chip8" | "chip-8" => Ok(Mode::Chip8),
            "schip" | "superchip" => Ok(Mode::SChip),
            "xo-chip" | "xochip" => Ok(Mode::XOChip),
            _ => Err(Error::InvalidMode {
                mode: s.to_string(),
            }),
        }
    }
}
