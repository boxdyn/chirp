//! Chirp: A chip-8 interpreter in Rust
//! Hello, world!

use chirp::{error::Result, prelude::*};
use gumdrop::*;
use minifb::*;
use std::fs::read;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Options, Hash)]
struct Arguments {
    #[options(help = "Print this help message.")]
    help: bool,
    #[options(help = "Enable behavior incompatible with modern software.")]
    pub authentic: bool,
    #[options(
        long = "break",
        help = "Set breakpoints for the emulator to stop at.",
        parse(try_from_str = "parse_hex")
    )]
    pub breakpoints: Vec<u16>,
    #[options(help = "Enable debug mode at startup.")]
    pub debug: bool,
    #[options(help = "Enable pause mode at startup.", default = "false")]
    pub pause: bool,
    #[options(help = "Load a ROM to run on Chirp.", required, free)]
    pub file: PathBuf,
    #[options(help = "Set the target framerate.", default = "60")]
    pub frame_rate: u64,
    #[options(help = "Set the instructions-per-frame rate.", default = "8")]
    pub speed: usize,
    #[options(help = "Run the emulator as fast as possible for `step` instructions.")]
    pub step: Option<usize>,
}

#[derive(Debug)]
struct State {
    pub speed: usize,
    pub step: Option<usize>,
    pub rate: u64,
    pub ch8: Chip8,
    pub win: Window,
    pub fb: FrameBuffer,
    pub ft: Instant,
}

impl State {
    fn new(options: Arguments) -> Result<Self> {
        let mut state = State {
            speed: options.speed,
            step: options.step,
            rate: options.frame_rate,
            ch8: Chip8 { bus: bus! {
                // Load the charset into ROM
                Charset [0x0050..0x00A0] = include_bytes!("mem/charset.bin"),
                // Load the ROM file into RAM
                Program [0x0200..0x1000] = &read(options.file)?,
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
                    authentic: options.authentic,
                    debug: options.debug,
                    pause: options.pause,
                    ..Default::default()
                },
            )},
            win: WindowBuilder::default().build()?,
            fb: FrameBuffer::new(64, 32),
            ft: Instant::now(),
        };
        state.fb.render(&mut state.win, &state.ch8.bus);
        Ok(state)
    }
    fn tick_cpu(&mut self) {
        if !self.ch8.cpu.flags.pause {
            let rate = self.speed;
            match self.step {
                Some(ticks) => {
                    self.ch8.cpu.multistep(&mut self.ch8.bus, ticks, rate);
                    // Pause the CPU and clear step
                    self.ch8.cpu.flags.pause = true;
                    self.step = None;
                },
                None => {
                    self.ch8.cpu.multistep(&mut self.ch8.bus, rate, rate);
                },
            }
        }
    }
    fn frame(&mut self) -> Option<()> {
        {
            if self.ch8.cpu.flags.pause {
                self.win.set_title("Chirp  ⏸")
            } else {
                self.win.set_title("Chirp  ▶");
            }
            // update framebuffer
            self.fb.render(&mut self.win, &mut self.ch8.bus);
            // get key input (has to happen after render)
            chirp::io::get_keys(&mut self.win, &mut self.ch8.cpu);
        }
        Some(())
    }
    fn keys(&mut self) -> Option<()> {
        // handle keybinds for the UI
        for key in self.win.get_keys_pressed(KeyRepeat::No) {
            use Key::*;
            match key {
                F1 | Comma => self.ch8.cpu.dump(),
                F2 | Period => self
                    .ch8.bus
                    .print_screen()
                    .expect("The 'screen' memory region exists"),
                F3 => {chirp::io::debug_dump_screen(&self.ch8.bus).expect("Unable to write debug screen dump"); eprintln!("Screen dumped to file.")},
                F4 | Slash => {
                    eprintln!(
                        "{}",
                        endis("Debug", {
                            self.ch8.cpu.flags.debug();
                            self.ch8.cpu.flags.debug
                        })
                    )
                }
                F5 | Backslash => eprintln!(
                    "{}",
                    endis("Pause", {
                        self.ch8.cpu.flags.pause();
                        self.ch8.cpu.flags.pause
                    })
                ),
                F6 | Enter => {
                    eprintln!("Step");
                    self.ch8.cpu.singlestep(&mut self.ch8.bus);
                }
                F7 => {
                    eprintln!("Set breakpoint {:x}", self.ch8.cpu.pc());
                    self.ch8.cpu.set_break(self.ch8.cpu.pc());
                }
                F8 => {
                    eprintln!("Unset breakpoint {:x}", self.ch8.cpu.pc());
                    self.ch8.cpu.unset_break(self.ch8.cpu.pc());
                }
                F9 | Delete => {
                    eprintln!("Soft reset state.cpu {:x}", self.ch8.cpu.pc());
                    self.ch8.cpu.soft_reset();
                    self.ch8.bus.clear_region(Screen);
                }
                F10 | Backspace => {
                    eprintln!("Hard reset state.cpu");
                    self.ch8.cpu = CPU::default();
                    self.ch8.bus.clear_region(Screen);
                }
                Escape => return None,
                _ => (),
            }
        }
        Some(())
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

/// Transforms a bool into "enabled"/"disabled"
fn endis(name: &str, state: bool) -> String {
    format!("{name} {}", if state { "enabled" } else { "disabled" })
}