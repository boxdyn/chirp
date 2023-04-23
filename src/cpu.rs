// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! Decodes and runs instructions

#[cfg(test)]
mod tests;

pub mod behavior;
pub mod bus;
pub mod flags;
pub mod instruction;
pub mod mode;
pub mod quirks;

use self::{
    bus::{Bus, Get, ReadWrite, Region::*},
    flags::Flags,
    instruction::{
        disassembler::{Dis, Disassembler},
        Insn,
    },
    mode::Mode,
    quirks::Quirks,
};
use crate::{
    bus,
    error::{Error, Result},
};
use imperative_rs::InstructionSet;
use owo_colors::OwoColorize;
use std::fmt::Debug;

type Reg = usize;
type Adr = u16;
type Nib = u8;

/// Represents the internal state of the CPU interpreter
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CPU {
    /// Flags that control how the CPU behaves, but which aren't inherent to the
    /// chip-8. Includes [Quirks], target IPF, etc.
    pub flags: Flags,
    // memory map info
    screen: Bus,
    font: Adr,
    // memory
    stack: Vec<Adr>,
    // registers
    pc: Adr,
    i: Adr,
    v: [u8; 16],
    delay: f64,
    sound: f64,
    // I/O
    keys: [bool; 16],
    // Execution data
    cycle: usize,
    breakpoints: Vec<Adr>,
    #[cfg_attr(feature = "serde", serde(skip))]
    disassembler: Dis,
}

// public interface
impl CPU {
    /// Constructs a new CPU, taking all configurable parameters
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let cpu = CPU::new(
    ///     0xf00,  // screen location
    ///     0x50,   // font location
    ///     0x200,  // start of program
    ///     Dis::default(),
    ///     vec![], // Breakpoints
    ///     Flags::default()
    /// );
    /// dbg!(cpu);
    /// ```
    pub fn new(
        rom: impl AsRef<std::path::Path>,
        font: Adr,
        pc: Adr,
        disassembler: Dis,
        breakpoints: Vec<Adr>,
        flags: Flags,
    ) -> Result<Self> {
        let mut cpu = CPU {
            disassembler,
            font,
            pc,
            breakpoints,
            flags,
            ..Default::default()
        };
        // load the provided rom
        cpu.load_program(rom)?;
        Ok(cpu)
    }

    /// Loads a program into the CPU's program space
    pub fn load_program(&mut self, rom: impl AsRef<std::path::Path>) -> Result<&mut Self> {
        self.load_program_bytes(&std::fs::read(rom)?)
    }

    /// Loads bytes into the CPU's program space
    pub fn load_program_bytes(&mut self, rom: &[u8]) -> Result<&mut Self> {
        self.screen.clear_region(Program);
        self.screen.load_region(Program, rom)?;
        Ok(self)
    }

    /// Presses a key, and reports whether the key's state changed.  
    /// If key does not exist, returns [Error::InvalidKey].
    ///
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    ///
    /// // press key `7`
    /// let did_press = cpu.press(0x7).unwrap();
    /// assert!(did_press);
    ///
    /// // press key `7` again, even though it's already pressed
    /// let did_press = cpu.press(0x7).unwrap();
    /// // it was already pressed, so nothing's changed.
    /// assert!(!did_press);
    /// ```
    pub fn press(&mut self, key: usize) -> Result<bool> {
        if let Some(keyref) = self.keys.get_mut(key) {
            if !*keyref {
                *keyref = true;
                return Ok(true);
            } // else do nothing
        } else {
            return Err(Error::InvalidKey { key });
        }
        Ok(false)
    }

    /// Releases a key, and reports whether the key's state changed.  
    /// If key is outside range `0..=0xF`, returns [Error::InvalidKey].
    ///
    /// If [Flags::keypause] was enabled, it is disabled,
    /// and the [Flags::lastkey] is recorded.
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// // press key `7`
    /// cpu.press(0x7).unwrap();
    /// // release key `7`
    /// let changed = cpu.release(0x7).unwrap();
    /// assert!(changed); // key released
    /// // try releasing `7` again
    /// let changed = cpu.release(0x7).unwrap();
    /// assert!(!changed); // key was not held
    /// ```
    pub fn release(&mut self, key: usize) -> Result<bool> {
        if let Some(keyref) = self.keys.get_mut(key) {
            if *keyref {
                *keyref = false;
                if self.flags.keypause {
                    self.flags.lastkey = Some(key);
                    self.flags.keypause = false;
                }
                return Ok(true);
            }
        } else {
            return Err(Error::InvalidKey { key });
        }
        Ok(false)
    }

    /// Sets a general purpose register in the CPU.  
    /// If the register doesn't exist, returns [Error::InvalidRegister]
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// // Create a new CPU, and set v4 to 0x41
    /// let mut cpu = CPU::default();
    /// cpu.set_v(0x4, 0x41).unwrap();
    /// // Dump the CPU registers
    /// cpu.dump();
    /// ```
    pub fn set_v(&mut self, reg: Reg, value: u8) -> Result<()> {
        if let Some(gpr) = self.v.get_mut(reg) {
            *gpr = value;
            Ok(())
        } else {
            Err(Error::InvalidRegister { reg })
        }
    }

    /// Gets a slice of the entire general purpose registers
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// // Create a new CPU, and set v4 to 0x41
    /// let mut cpu = CPU::default();
    /// cpu.set_v(0x0, 0x41);
    /// assert_eq!(
    ///     cpu.v(),
    ///     [0x41, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    /// )
    /// ```
    pub fn v(&self) -> &[u8] {
        self.v.as_slice()
    }

    /// Gets the program counter
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// assert_eq!(0x200, cpu.pc());
    /// ```
    pub fn pc(&self) -> Adr {
        self.pc
    }

    /// Gets the I register
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// assert_eq!(0, cpu.i());
    /// ```
    pub fn i(&self) -> Adr {
        self.i
    }

    /// Gets the value in the Sound Timer register
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// assert_eq!(0, cpu.sound());
    /// ```
    pub fn sound(&self) -> u8 {
        self.sound as u8
    }

    /// Gets the value in the Delay Timer register
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// assert_eq!(0, cpu.delay());
    /// ```
    pub fn delay(&self) -> u8 {
        self.delay as u8
    }

    /// Gets the number of cycles the CPU has executed
    ///
    /// If cpu.flags.monotonic is Some, the cycle count will be
    /// updated even when the CPU is in drawpause or keypause
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// assert_eq!(0x0, cpu.cycle());
    /// ```
    pub fn cycle(&self) -> usize {
        self.cycle
    }

    /// Soft resets the CPU, releasing keypause and
    /// reinitializing the program counter to 0x200
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::new(
    ///     0xf00,
    ///     0x50,
    ///     0x340,
    ///     Dis::default(),
    ///     vec![],
    ///     Flags::default()
    /// );
    /// cpu.flags.keypause = true;
    /// cpu.flags.draw_wait = true;
    /// assert_eq!(0x340, cpu.pc());
    /// cpu.soft_reset();
    /// assert_eq!(0x200, cpu.pc());
    /// assert_eq!(false, cpu.flags.keypause);
    /// assert_eq!(false, cpu.flags.draw_wait);
    /// ```
    pub fn soft_reset(&mut self) {
        self.pc = 0x200;
        self.flags.keypause = false;
        self.flags.draw_wait = false;
    }

    /// Resets the emulator.
    ///
    /// Touches the [Flags] (keypause, draw_wait, draw_mode, and lastkey),
    /// stack, pc, registers, keys, and cycle count.
    ///
    /// Does not touch [Quirks], [Mode], [Dis], breakpoints, or memory map.
    pub fn reset(&mut self) {
        self.flags = Flags {
            keypause: false,
            draw_wait: false,
            draw_mode: false,
            lastkey: None,
            ..self.flags
        };
        // clear the stack
        self.stack.truncate(0);
        // Reset the program counter
        self.pc = 0x200;
        // Zero the registers
        self.i = 0;
        self.v = [0; 16];
        self.delay = 0.0;
        self.sound = 0.0;
        // I/O
        self.keys = [false; 16];
        // Execution data
        self.cycle = 0;
    }

    /// Set a breakpoint
    // TODO: Unit test this
    pub fn set_break(&mut self, point: Adr) -> &mut Self {
        if !self.breakpoints.contains(&point) {
            self.breakpoints.push(point)
        }
        self
    }

    /// Unset a breakpoint
    // TODO: Unit test this
    pub fn unset_break(&mut self, point: Adr) -> &mut Self {
        fn linear_find(needle: Adr, haystack: &[Adr]) -> Option<usize> {
            for (i, v) in haystack.iter().enumerate() {
                if *v == needle {
                    return Some(i);
                }
            }
            None
        }
        if let Some(idx) = linear_find(point, self.breakpoints.as_slice()) {
            assert_eq!(point, self.breakpoints.swap_remove(idx));
        }
        self
    }

    /// Gets a slice of breakpoints
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// assert_eq!(cpu.breakpoints(), &[]);
    /// ```
    pub fn breakpoints(&self) -> &[Adr] {
        self.breakpoints.as_slice()
    }

    /// Unpauses the emulator for a single tick,
    /// even if cpu.flags.pause is set.
    ///
    /// Like with [CPU::tick], this returns [Error::UnimplementedInstruction]
    /// if the instruction is unimplemented.
    ///
    /// NOTE: does not synchronize with delay timers
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// let mut bus = bus!{
    ///     Program [0x0200..0x0f00] = &[
    ///         0x00, 0xe0, // cls
    ///         0x22, 0x02, // jump 0x202 (pc)
    ///     ],
    ///     Screen  [0x0f00..0x1000],
    /// };
    /// cpu.singlestep(&mut bus).unwrap();
    /// assert_eq!(0x202, cpu.pc());
    /// assert_eq!(1, cpu.cycle());
    /// ```
    pub fn singlestep(&mut self, bus: &mut Bus) -> Result<&mut Self> {
        self.flags.pause = false;
        self.tick(bus)?;
        self.flags.draw_wait = false;
        self.flags.pause = true;
        Ok(self)
    }

    /// Unpauses the emulator for `steps` ticks
    ///
    /// Ticks the timers every `rate` ticks
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// let mut bus = bus!{
    ///     Program [0x0200..0x0f00] = &[
    ///         0x00, 0xe0, // cls
    ///         0x22, 0x02, // jump 0x202 (pc)
    ///     ],
    ///     Screen  [0x0f00..0x1000],
    /// };
    /// cpu.multistep(&mut bus, 0x20)
    ///     .expect("The program should only have valid opcodes.");
    /// assert_eq!(0x202, cpu.pc());
    /// assert_eq!(0x20, cpu.cycle());
    /// ```
    pub fn multistep(&mut self, screen: &mut Bus, steps: usize) -> Result<&mut Self> {
        for _ in 0..steps {
            self.tick(screen)?;
            let speed = 1.0 / steps as f64;
            self.delay -= speed;
            self.sound -= speed;
        }
        self.flags.draw_wait = false;
        Ok(self)
    }

    /// Executes a single instruction
    ///
    /// Returns [Error::BreakpointHit] if a breakpoint was hit after the instruction executed.  
    /// This result contains information about the breakpoint, but can be safely ignored.
    ///
    /// Returns [Error::UnimplementedInstruction] if the instruction at `pc` is unimplemented.
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// let mut bus = bus!{
    ///     Program [0x0200..0x0f00] = &[
    ///         0x00, 0xe0, // cls
    ///         0x22, 0x02, // jump 0x202 (pc)
    ///     ],
    ///     Screen  [0x0f00..0x1000],
    /// };
    /// cpu.tick(&mut bus)
    ///     .expect("0x00e0 (cls) should be a valid opcode.");
    /// assert_eq!(0x202, cpu.pc());
    /// assert_eq!(1, cpu.cycle());
    /// ```
    /// Returns [Error::UnimplementedInstruction] if the instruction is not implemented.
    /// ```rust
    /// # use chirp::*;
    /// # use chirp::error::Error;
    /// let mut cpu = CPU::default();
    /// # cpu.flags.debug = true;        // enable live disassembly
    /// # cpu.flags.monotonic = Some(8); // enable monotonic/test timing
    /// let mut bus = bus!{
    ///     Program [0x0200..0x0f00] = &[
    ///         0xff, 0xff, // invalid!
    ///         0x22, 0x02, // jump 0x202 (pc)
    ///     ],
    ///     Screen  [0x0f00..0x1000],
    /// };
    /// dbg!(cpu.tick(&mut bus))
    ///     .expect_err("Should return Error::InvalidInstruction { 0xffff }");
    /// ```
    pub fn tick(&mut self, screen: &mut Bus) -> Result<&mut Self> {
        // Do nothing if paused
        if self.flags.is_paused() {
            // always tick in test mode
            if self.flags.monotonic.is_some() {
                self.cycle += 1;
            }
            return Ok(self);
        }
        self.cycle += 1;
        let opchunk = self
            .screen
            .get(self.pc as usize..)
            .ok_or(Error::InvalidAddressRange {
                range: (self.pc as usize..).into(),
            })?;
        // fetch opcode
        let opcode: &[u8; 2] =
            if let Some(slice) = self.screen.get(self.pc as usize..self.pc as usize + 2) {
                slice
                    .try_into()
                    .expect("`slice` should be exactly 4 bytes.")
            } else {
                return Err(Error::InvalidAddressRange {
                    range: (self.pc as usize..self.pc as usize + 4).into(),
                });
            };

        // Print opcode disassembly:
        if self.flags.debug {
            std::println!(
                "{:3} {:03x}: {:<36}",
                self.cycle.bright_black(),
                self.pc,
                self.disassembler.once(u16::from_be_bytes(*opcode))
            );
        }

        // decode opcode
        if let Ok((inc, insn)) = Insn::decode(opchunk) {
            self.pc = self.pc.wrapping_add(inc as u16);
            self.execute(screen, insn);
        } else {
            return Err(Error::UnimplementedInstruction {
                word: u16::from_be_bytes(*opcode),
            });
        }

        // process breakpoints
        if !self.breakpoints.is_empty() && self.breakpoints.contains(&self.pc) {
            self.flags.pause = true;
            return Err(Error::BreakpointHit {
                addr: self.pc,
                next: self.screen.read(self.pc),
            });
        }
        Ok(self)
    }

    /// Dumps the current state of all CPU registers, and the cycle count
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// cpu.dump();
    /// ```
    /// outputs
    /// ```text
    /// PC: 0200, SP: 0efe, I: 0000
    /// v0: 00 v1: 00 v2: 00 v3: 00
    /// v4: 00 v5: 00 v6: 00 v7: 00
    /// v8: 00 v9: 00 vA: 00 vB: 00
    /// vC: 00 vD: 00 vE: 00 vF: 00
    /// DLY: 0, SND: 0, CYC:      0
    /// ```
    pub fn dump(&self) {
        //let dumpstyle = owo_colors::Style::new().bright_black();
        std::println!(
            "PC: {:04x}, SP: {:04x}, I: {:04x}\n{}DLY: {}, SND: {}, CYC: {:6}",
            self.pc,
            self.stack.len(),
            self.i,
            self.v
                .into_iter()
                .enumerate()
                .map(|(i, gpr)| {
                    format!(
                        "v{i:X}: {gpr:02x} {}",
                        match i % 4 {
                            3 => "\n",
                            _ => "",
                        }
                    )
                })
                .collect::<String>(),
            self.delay as u8,
            self.sound as u8,
            self.cycle,
        );
    }
}

impl Debug for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CPU")
            .field("flags", &self.flags)
            .field("font", &self.font)
            .field("stack", &self.stack)
            .field("pc", &self.pc)
            .field("i", &self.i)
            .field("v", &self.v)
            .field("delay", &self.delay)
            .field("sound", &self.sound)
            .field("keys", &self.keys)
            .field("cycle", &self.cycle)
            .field("breakpoints", &self.breakpoints)
            .field("disassembler", &self.disassembler)
            .finish_non_exhaustive()
    }
}

impl Default for CPU {
    /// Constructs a new CPU with sane defaults and debug mode ON
    ///
    /// | value  | default | description
    /// |--------|---------|------------
    /// | screen |`0x0f00` | Location of screen memory.
    /// | font   |`0x0050` | Location of font memory.
    /// | pc     |`0x0200` | Start location. Generally 0x200 or 0x600.
    ///
    /// # Examples
    /// ```rust
    /// use chirp::*;
    /// let mut cpu = CPU::default();
    /// ```
    fn default() -> Self {
        CPU {
            stack: vec![],
            screen: bus! {
                Charset [0x0050..0x00a0] = include_bytes!("mem/charset.bin"),
                Program [0x0200..0x1000],
            },
            font: 0x050,
            pc: 0x200,
            i: 0,
            v: [0; 16],
            delay: 0.0,
            sound: 0.0,
            cycle: 0,
            keys: [false; 16],
            flags: Flags {
                debug: true,
                ..Default::default()
            },
            breakpoints: vec![],
            disassembler: Dis::default(),
        }
    }
}
