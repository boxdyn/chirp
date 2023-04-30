// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! Chirp: A chip-8 interpreter in Rust
//! Hello, world!

#[cfg(test)]
mod tests;
mod ui;

use chirp::error::Error::BreakpointHit;
use chirp::{error::Result, *};
use gumdrop::*;
use owo_colors::OwoColorize;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use ui::*;

pub fn main() -> Result<()> {
    let options = Arguments::parse_args_default_or_exit();
    let state = State::new(options)?;
    for result in state {
        if let Err(e) = result {
            eprintln!("{}", e.bold().red());
            break;
        }
    }
    Ok(())
}

/// Parses a hexadecimal string into a u16
fn parse_hex(value: &str) -> std::result::Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(value, 16)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Options, Hash)]
struct Arguments {
    #[options(help = "Load a ROM to run on Chirp.", required, free)]
    pub file: PathBuf,
    #[options(help = "Print this help message.")]
    help: bool,
    #[options(help = "Enable debug mode at startup.")]
    pub debug: bool,
    #[options(help = "Enable pause mode at startup.")]
    pub pause: bool,

    #[options(help = "Set the instructions-per-delay rate. If unspecified, use realtime.")]
    pub speed: Option<usize>,
    #[options(help = "Set the instructions-per-frame rate.")]
    pub step: Option<usize>,
    #[options(help = "Enable performance benchmarking on stderr (requires -S)")]
    pub perf: bool,

    #[options(
        help = "Run in (Chip8, SChip, XOChip) mode.",
        //parse(from_str = "parse_mode")
    )]
    pub mode: Option<Mode>,

    #[options(
        short = "z",
        help = "Disable setting vF to 0 after a bitwise operation."
    )]
    pub vfreset: bool,
    #[options(
        short = "x",
        help = "Disable waiting for vblank after issuing a draw call."
    )]
    pub drawsync: bool,

    #[options(
        short = "c",
        help = "Use CHIP-48 style DMA instructions, which don't touch I."
    )]
    pub memory: bool,

    #[options(
        short = "v",
        help = "Use CHIP-48 style bit-shifts, which don't touch vY."
    )]
    pub shift: bool,

    #[options(
        short = "b",
        help = "Use SUPER-CHIP style indexed jump, which is indexed relative to v[adr]."
    )]
    pub jumping: bool,

    #[options(
        long = "break",
        help = "Set breakpoints for the emulator to stop at.",
        parse(try_from_str = "parse_hex"),
        meta = "BP"
    )]
    pub breakpoints: Vec<u16>,

    #[options(
        help = "Load additional word at address 0x1fe",
        parse(try_from_str = "parse_hex"),
        meta = "WORD"
    )]
    pub data: u16,

    #[options(help = "Set the target framerate.", default = "60", meta = "FR")]
    pub frame_rate: u64,
}

#[derive(Debug)]
pub struct Chip8 {
    pub cpu: CPU,
    pub screen: Screen,
}
#[derive(Debug)]
struct State {
    pub speed: usize,
    pub step: Option<usize>,
    pub rate: u64,
    pub perf: bool,
    pub ch8: Chip8,
    pub ui: UI,
    pub ft: Instant,
}

impl State {
    fn new(options: Arguments) -> Result<Self> {
        let mut state = State {
            speed: options.speed.unwrap_or(8),
            step: options.step,
            rate: options.frame_rate,
            perf: options.perf,
            ch8: Chip8 {
                cpu: CPU::new(
                    Some(&options.file),
                    0x50,
                    0x200,
                    Dis::default(),
                    options.breakpoints,
                    Flags {
                        quirks: options.mode.unwrap_or_default().into(),
                        debug: options.debug,
                        pause: options.pause,
                        ..Default::default()
                    },
                )?,
                screen: Screen::default(),
            },
            ui: UIBuilder::new(128, 64, &options.file).build()?,
            ft: Instant::now(),
        };
        // Flip the state of the quirks
        state.ch8.cpu.flags.quirks.bin_ops ^= options.vfreset;
        state.ch8.cpu.flags.quirks.dma_inc ^= options.memory;
        state.ch8.cpu.flags.quirks.draw_wait ^= options.drawsync;
        state.ch8.cpu.flags.quirks.shift ^= options.shift;
        state.ch8.cpu.flags.quirks.stupid_jumps ^= options.jumping;
        state.ch8.screen.write(0x1feu16, options.data);
        Ok(state)
    }
    fn keys(&mut self) -> Result<bool> {
        self.ui.keys(&mut self.ch8)
    }
    fn frame(&mut self) -> Result<bool> {
        self.ui.frame(&mut self.ch8)
    }
    fn tick_cpu(&mut self) -> Result<()> {
        if !self.ch8.cpu.flags.pause {
            let rate = self.speed;
            match self.step {
                Some(ticks) => {
                    let time = Instant::now();
                    self.ch8.cpu.multistep(&mut self.ch8.screen, ticks)?;
                    if self.perf {
                        let time = time.elapsed();
                        let nspt = time.as_secs_f64() / ticks as f64;
                        eprintln!(
                            "{ticks},\t{time:.05?},\t{:.4} nspt,\t{} ipf,\t{} mips",
                            nspt * 1_000_000_000.0,
                            ((1.0 / 60.0f64) / nspt).trunc(),
                            (1.0 / nspt).trunc() / 1_000_000.0,
                        );
                    }
                }
                None => {
                    self.ch8.cpu.multistep(&mut self.ch8.screen, rate)?;
                }
            }
        }
        Ok(())
    }
    fn wait_for_next_frame(&mut self) {
        let rate = Duration::from_nanos(1_000_000_000 / self.rate + 1);
        std::thread::sleep(rate.saturating_sub(self.ft.elapsed()));
        self.ft += rate;
    }
}

impl Iterator for State {
    type Item = Result<()>;

    /// Pretty heavily abusing iterators here, in an annoying way
    fn next(&mut self) -> Option<Self::Item> {
        self.wait_for_next_frame();
        match self.keys() {
            Ok(opt) if !opt => return None,
            Err(e) => return Some(Err(e)), // summary lol
            _ => (),
        }
        // Allow breakpoint hit messages
        match self.tick_cpu() {
            Err(BreakpointHit { addr, next }) => {
                eprintln!("Breakpoint hit: {:3x} ({:4x})", addr, next);
            }
            Err(e) => return Some(Err(e)),
            _ => (),
        }
        match self.frame() {
            Ok(opt) if !opt => return None,
            Err(e) => return Some(Err(e)),
            _ => (),
        }
        Some(Ok(()))
    }
}
