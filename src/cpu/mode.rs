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
            "chip" | "chip8" | "chip-8" => Ok(Mode::Chip8),
            "s" | "schip" | "superchip" | "super chip" => Ok(Mode::SChip),
            "xo" | "xochip" | "xo-chip" => Ok(Mode::XOChip),
            _ => Err(Error::InvalidMode { mode: s.into() }),
        }
    }
}

impl AsRef<str> for Mode {
    fn as_ref(&self) -> &str {
        match self {
            Mode::Chip8 => "Chip-8",
            Mode::SChip => "Super Chip",
            Mode::XOChip => "XO-Chip",
        }
    }
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        self.as_ref().into()
    }
}

impl From<usize> for Mode {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Chip8,
            1 => Self::SChip,
            2 => Self::XOChip,
            _ => Self::Chip8,
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
