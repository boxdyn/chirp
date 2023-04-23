// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! Selects the memory behavior of the [super::CPU]
//!
//! Since [Quirks] implements [`From<Mode>`],
//! this can be used to select the appropriate quirk-set

use super::Quirks;
use crate::error::Error;
use std::str::FromStr;

/// Selects the memory behavior of the interpreter
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mode {
    /// VIP emulation mode
    #[default]
    Chip8,
    /// Super Chip emulation mode
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

impl From<Mode> for Quirks {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Chip8 => false.into(),
            Mode::SChip => true.into(),
            Mode::XOChip => Self {
                bin_ops: true,
                shift: false,
                draw_wait: true,
                screen_wrap: true,
                dma_inc: false,
                stupid_jumps: false,
            },
        }
    }
}
