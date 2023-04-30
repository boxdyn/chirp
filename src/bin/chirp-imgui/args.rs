//! Parses arguments into a struct

use std::path::PathBuf;

use super::Emulator;
use chirp::Mode;
use gumdrop::*;

/// Parses a hexadecimal string into a u16
fn parse_hex(value: &str) -> std::result::Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(value, 16)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Options, Hash)]
pub struct Arguments {
    #[options(help = "Load a ROM to run on Chirp.", required, free)]
    pub file: PathBuf,
    #[options(help = "Print this help message.")]
    help: bool,
    #[options(help = "Enable debug mode at startup.")]
    pub debug: bool,
    #[options(help = "Enable pause mode at startup.")]
    pub pause: bool,
    #[options(help = "Set the instructions-per-frame rate.")]
    pub speed: Option<usize>,
    // #[options(help = "Enable performance benchmarking on stderr (requires -S)")]
    // pub perf: bool,
    #[options(help = "Run in (Chip8, SChip, XOChip) mode.")]
    pub mode: Option<Mode>,
    #[options(help = "Set the target framerate.", default = "60", meta = "FR")]
    pub frame_rate: u64,
}

impl Arguments {
    pub fn parse() -> Arguments {
        Arguments::parse_args_default_or_exit()
    }
}

impl From<Arguments> for Emulator {
    fn from(value: Arguments) -> Self {
        let mut emu = Emulator::new(value.speed.unwrap_or(10), value.file);
        if let Some(mode) = value.mode {
            emu.set_quirks(mode.into());
        }
        if value.pause {
            emu.pause()
        }
        emu.set_disasm(value.debug);
        emu
    }
}
