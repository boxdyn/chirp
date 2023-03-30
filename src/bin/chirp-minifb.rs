// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE.txt for details)

//! Chirp: A chip-8 interpreter in Rust
//! Hello, world!

use chirp::{error::Result, prelude::*};
use gumdrop::*;
use std::fs::read;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

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

    #[options(help = "Set the instructions-per-frame rate.")]
    pub speed: Option<usize>,
    #[options(help = "Run the emulator as fast as possible for `step` instructions.")]
    pub step: Option<usize>,

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
struct State {
    pub speed: usize,
    pub step: Option<usize>,
    pub rate: u64,
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
            ch8: Chip8 {
                bus: bus! {
                    // Load the charset into ROM
                    Charset [0x0050..0x00A0] = include_bytes!("../mem/charset.bin"),
                    // Load the ROM file into RAM
                    Program [0x0200..0x1000] = &read(&options.file)?,
                    // Create a screen
                    Screen  [0x1000..0x1100],
                    // Create a stack
                    Stack   [0x0EA0..0x0F00],
                },
                cpu: CPU::new(
                    0x1000,
                    0x50,
                    0x200,
                    0xefe,
                    Disassemble::default(),
                    options.breakpoints,
                    ControlFlags {
                        quirks: chirp::cpu::Quirks {
                            bin_ops: !options.vfreset,
                            shift: !options.shift,
                            draw_wait: !options.drawsync,
                            dma_inc: !options.memory,
                            stupid_jumps: options.jumping,
                        },
                        debug: options.debug,
                        pause: options.pause,
                        monotonic: options.speed,
                        ..Default::default()
                    },
                ),
            },
            ui: UIBuilder::default().rom(&options.file).build()?,
            ft: Instant::now(),
        };
        state.ch8.bus.write(0x1feu16, options.data);
        Ok(state)
    }
    fn keys(&mut self) -> Option<()> {
        self.ui.keys(&mut self.ch8)
    }
    fn frame(&mut self) -> Option<()> {
        self.ui.frame(&mut self.ch8)
    }
    fn tick_cpu(&mut self) {
        if !self.ch8.cpu.flags.pause {
            let rate = self.speed;
            match self.step {
                Some(ticks) => {
                    self.ch8.cpu.multistep(&mut self.ch8.bus, ticks);
                    // Pause the CPU and clear step
                    self.ch8.cpu.flags.pause = true;
                    self.step = None;
                }
                None => {
                    self.ch8.cpu.multistep(&mut self.ch8.bus, rate);
                }
            }
        }
    }
    fn wait_for_next_frame(&mut self) {
        let rate = 1_000_000_000 / self.rate + 1;
        std::thread::sleep(Duration::from_nanos(rate).saturating_sub(self.ft.elapsed()));
        self.ft = Instant::now();
    }
}

impl Iterator for State {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.wait_for_next_frame();
        self.keys()?;
        self.tick_cpu();
        self.frame();
        Some(())
    }
}

fn main() -> Result<()> {
    let options = Arguments::parse_args_default_or_exit();
    let state = State::new(options)?;
    Ok(for _ in state {})
}

/// Parses a hexadecimal string into a u16
fn parse_hex(value: &str) -> std::result::Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(value, 16)
}