// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE.txt for details)

//! Decodes and runs instructions

#[cfg(test)]
mod tests;

pub mod disassemble;

use self::disassemble::Disassemble;
use crate::bus::{Bus, Read, Region, Write};
use owo_colors::OwoColorize;
use rand::random;
use std::time::Instant;

type Reg = usize;
type Adr = u16;
type Nib = u8;

/// Controls the authenticity behavior of the CPU on a granular level.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quirks {
    /// Binary ops in `8xy`(`1`, `2`, `3`) should set vF to 0
    pub bin_ops: bool,
    /// Shift ops in `8xy`(`6`, `E`) should source from vY instead of vX
    pub shift: bool,
    /// Draw operations should pause execution until the next timer tick
    pub draw_wait: bool,
    /// DMA instructions `Fx55`/`Fx65` should change I to I + x + 1
    pub dma_inc: bool,
    /// Indexed jump instructions should go to ADR + v`N` where `N` is high nibble of adr
    pub stupid_jumps: bool,
}

impl From<bool> for Quirks {
    fn from(value: bool) -> Self {
        if value {
            Quirks {
                bin_ops: true,
                shift: true,
                draw_wait: true,
                dma_inc: true,
                stupid_jumps: false,
            }
        } else {
            Quirks {
                bin_ops: false,
                shift: false,
                draw_wait: false,
                dma_inc: false,
                stupid_jumps: false,
            }
        }
    }
}

impl Default for Quirks {
    fn default() -> Self {
        Self::from(false)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlFlags {
    pub debug: bool,
    pub pause: bool,
    pub keypause: bool,
    pub vbi_wait: bool,
    pub lastkey: Option<usize>,
    pub quirks: Quirks,
    pub monotonic: Option<usize>,
}

impl ControlFlags {
    /// Toggles debug mode
    ///
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     assert_eq!(true, cpu.flags.debug);
    ///     cpu.flags.debug();
    ///     assert_eq!(false, cpu.flags.debug);
    ///#    Ok(())
    ///# }
    /// ```
    pub fn debug(&mut self) {
        self.debug = !self.debug
    }

    /// Toggles pause
    ///
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     assert_eq!(false, cpu.flags.pause);
    ///     cpu.flags.pause();
    ///     assert_eq!(true, cpu.flags.pause);
    ///#    Ok(())
    ///# }
    /// ```
    pub fn pause(&mut self) {
        self.pause = !self.pause
    }
}

/// Represents the internal state of the CPU interpreter
#[derive(Clone, Debug, PartialEq)]
pub struct CPU {
    pub flags: ControlFlags,
    // memory map info
    screen: Adr,
    font: Adr,
    // registers
    pc: Adr,
    sp: Adr,
    i: Adr,
    v: [u8; 16],
    delay: f64,
    sound: f64,
    // I/O
    keys: [bool; 16],
    // Execution data
    timer: Instant,
    cycle: usize,
    breakpoints: Vec<Adr>,
    disassembler: Disassemble,
}

// public interface
impl CPU {
    // TODO: implement From<&bus> for CPU
    /// Constructs a new CPU, taking all configurable parameters
    /// # Examples
    /// ```rust
    /// # use chirp::prelude::*;
    /// let cpu = CPU::new(
    ///     0xf00,  // screen location
    ///     0x50,   // font location
    ///     0x200,  // start of program
    ///     0xefe,  // top of stack
    ///     Disassemble::default(),
    ///     vec![], // Breakpoints
    ///     ControlFlags::default()
    /// );
    /// dbg!(cpu);
    /// ```
    pub fn new(
        screen: Adr,
        font: Adr,
        pc: Adr,
        sp: Adr,
        disassembler: Disassemble,
        breakpoints: Vec<Adr>,
        flags: ControlFlags,
    ) -> Self {
        CPU {
            disassembler,
            screen,
            font,
            pc,
            sp,
            breakpoints,
            flags,
            ..Default::default()
        }
    }

    /// Presses a key
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     cpu.press(0x7);
    ///     cpu.press(0xF);
    ///#    Ok(())
    ///# }
    /// ```
    pub fn press(&mut self, key: usize) {
        if (0..16).contains(&key) {
            self.keys[key] = true;
        }
    }

    /// Releases a key
    ///
    /// If keypause is enabled, this disables keypause and records the last released key.
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     cpu.press(0x7);
    ///     cpu.release(0x7);
    ///#    Ok(())
    ///# }
    /// ```
    pub fn release(&mut self, key: usize) {
        if (0..16).contains(&key) {
            self.keys[key] = false;
            if self.flags.keypause {
                self.flags.lastkey = Some(key);
            }
            self.flags.keypause = false;
        }
    }

    /// Sets a general purpose register in the CPU
    /// # Examples
    /// ```rust
    /// # use chirp::prelude::*;
    /// // Create a new CPU, and set v4 to 0x41
    /// let mut cpu = CPU::default();
    /// cpu.set_v(0x4, 0x41);
    /// // Dump the CPU registers
    /// cpu.dump();
    /// ```
    pub fn set_v(&mut self, gpr: Reg, value: u8) {
        if let Some(gpr) = self.v.get_mut(gpr) {
            *gpr = value;
        }
    }

    /// Gets a slice of the entire general purpose register field
    /// # Examples
    /// ```rust
    /// # use chirp::prelude::*;
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
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     assert_eq!(0x200, cpu.pc());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn pc(&self) -> Adr {
        self.pc
    }

    /// Gets the I register
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     assert_eq!(0, cpu.i());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn i(&self) -> Adr {
        self.i
    }

    /// Gets the value in the Sound Timer register
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     assert_eq!(0, cpu.sound());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn sound(&self) -> u8 {
        self.sound as u8
    }

    /// Gets the value in the Delay Timer register
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     assert_eq!(0, cpu.delay());
    ///#    Ok(())
    ///# }
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
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     assert_eq!(0x0, cpu.cycle());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn cycle(&self) -> usize {
        self.cycle
    }

    /// Soft resets the CPU, releasing keypause and
    /// reinitializing the program counter to 0x200
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::new(
    ///         0xf00,
    ///         0x50,
    ///         0x340,
    ///         0xefe,
    ///         Disassemble::default(),
    ///         vec![],
    ///         ControlFlags::default()
    ///     );
    ///     cpu.flags.keypause = true;
    ///     cpu.flags.vbi_wait = true;
    ///     assert_eq!(0x340, cpu.pc());
    ///     cpu.soft_reset();
    ///     assert_eq!(0x200, cpu.pc());
    ///     assert_eq!(false, cpu.flags.keypause);
    ///     assert_eq!(false, cpu.flags.vbi_wait);
    ///#    Ok(())
    ///# }
    /// ```
    pub fn soft_reset(&mut self) {
        self.pc = 0x200;
        self.flags.keypause = false;
        self.flags.vbi_wait = false;
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
        fn linear_find(needle: Adr, haystack: &Vec<Adr>) -> Option<usize> {
            for (i, v) in haystack.iter().enumerate() {
                if *v == needle {
                    return Some(i);
                }
            }
            None
        }
        if let Some(idx) = linear_find(point, &self.breakpoints) {
            assert_eq!(point, self.breakpoints.swap_remove(idx));
        }
        self
    }

    /// Gets a slice of breakpoints
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     assert_eq!(cpu.breakpoints(), &[]);
    ///#    Ok(())
    ///# }
    /// ```
    pub fn breakpoints(&self) -> &[Adr] {
        &self.breakpoints.as_slice()
    }

    /// Unpauses the emulator for a single tick,
    /// even if cpu.flags.pause is set.
    ///
    /// NOTE: does not synchronize with delay timers
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     let mut bus = bus!{
    ///         Program [0x0200..0x0f00] = &[
    ///             0x00, 0xe0, // cls
    ///             0x22, 0x02, // jump 0x202 (pc)
    ///         ],
    ///         Screen  [0x0f00..0x1000],
    ///     };
    ///     cpu.singlestep(&mut bus);
    ///     assert_eq!(0x202, cpu.pc());
    ///     assert_eq!(1, cpu.cycle());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn singlestep(&mut self, bus: &mut Bus) -> &mut Self {
        self.flags.pause = false;
        self.tick(bus);
        self.flags.vbi_wait = false;
        self.flags.pause = true;
        self
    }

    /// Unpauses the emulator for `steps` ticks
    ///
    /// Ticks the timers every `rate` ticks
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     let mut bus = bus!{
    ///         Program [0x0200..0x0f00] = &[
    ///             0x00, 0xe0, // cls
    ///             0x22, 0x02, // jump 0x202 (pc)
    ///         ],
    ///         Screen  [0x0f00..0x1000],
    ///     };
    ///     cpu.multistep(&mut bus, 0x20);
    ///     assert_eq!(0x202, cpu.pc());
    ///     assert_eq!(0x20, cpu.cycle());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn multistep(&mut self, bus: &mut Bus, steps: usize) -> &mut Self {
        for _ in 0..steps {
            self.tick(bus);
            self.vertical_blank();
        }
        self
    }

    /// Simulates vertical blanking
    ///
    /// If monotonic timing is `enabled`:
    /// - Ticks the sound and delay timers according to CPU cycle count
    /// - Disables framepause
    /// If monotonic timing is `disabled`:
    /// - Subtracts the elapsed time in fractions of a frame
    ///   from st/dt
    /// - Disables framepause if the duration exceeds that of a frame
    pub fn vertical_blank(&mut self) -> &mut Self {
        if self.flags.pause {
            return self;
        }
        // Use a monotonic counter when testing
        if let Some(speed) = self.flags.monotonic {
            if self.flags.vbi_wait {
                self.flags.vbi_wait = !(self.cycle % speed == 0);
            }
            self.delay -= 1.0 / speed as f64;
            self.sound -= 1.0 / speed as f64;
            return self;
        };

        let time = self.timer.elapsed().as_secs_f64() * 60.0;
        self.timer = Instant::now();
        if time > 1.0 {
            self.flags.vbi_wait = false;
        }
        if self.delay > 0.0 {
            self.delay -= time;
        }
        if self.sound > 0.0 {
            self.sound -= time;
        }
        self
    }

    /// Executes a single instruction
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     let mut bus = bus!{
    ///         Program [0x0200..0x0f00] = &[
    ///             0x00, 0xe0, // cls
    ///             0x22, 0x02, // jump 0x202 (pc)
    ///         ],
    ///         Screen  [0x0f00..0x1000],
    ///     };
    ///     cpu.tick(&mut bus);
    ///     assert_eq!(0x202, cpu.pc());
    ///     assert_eq!(1, cpu.cycle());
    ///#    Ok(())
    ///# }
    /// ```
    /// # Panics
    /// Will panic if an invalid instruction is executed
    /// ```rust,should_panic
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///#     cpu.flags.debug = true;        // enable live disassembly
    ///#     cpu.flags.monotonic = Some(8); // enable monotonic/test timing
    ///     let mut bus = bus!{
    ///         Program [0x0200..0x0f00] = &[
    ///             0xff, 0xff, // invalid!
    ///             0x22, 0x02, // jump 0x202 (pc)
    ///         ],
    ///         Screen  [0x0f00..0x1000],
    ///     };
    ///     cpu.multistep(&mut bus, 0x10); // panics!
    ///#    Ok(())
    ///# }
    /// ```
    pub fn tick(&mut self, bus: &mut Bus) -> &mut Self {
        // Do nothing if paused
        if self.flags.pause || self.flags.vbi_wait || self.flags.keypause {
            // always tick in test mode
            if self.flags.monotonic.is_some() {
                self.cycle += 1;
            }
            return self;
        }
        self.cycle += 1;
        // fetch opcode
        let opcode: u16 = bus.read(self.pc);
        let pc = self.pc;

        // DINC pc
        self.pc = self.pc.wrapping_add(2);
        // decode opcode

        use disassemble::{a, b, i, n, x, y};
        let (i, x, y, n, b, a) = (
            i(opcode),
            x(opcode),
            y(opcode),
            n(opcode),
            b(opcode),
            a(opcode),
        );
        match i {
            // # Issue a system call
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | 00e0 | Clear screen memory to all 0       |
            // | 00ee | Return from subroutine             |
            0x0 => match a {
                0x0e0 => self.clear_screen(bus),
                0x0ee => self.ret(bus),
                _ => self.sys(a),
            },
            // | 1aaa | Sets pc to an absolute address
            0x1 => self.jump(a),
            // | 2aaa | Pushes pc onto the stack, then jumps to a
            0x2 => self.call(a, bus),
            // | 3xbb | Skips next instruction if register X == b
            0x3 => self.skip_equals_immediate(x, b),
            // | 4xbb | Skips next instruction if register X != b
            0x4 => self.skip_not_equals_immediate(x, b),
            // # Performs a register-register comparison
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | 9XY0 | Skip next instruction if vX == vY  |
            0x5 => match n {
                0x0 => self.skip_equals(x, y),
                _ => self.unimplemented(opcode),
            },
            // 6xbb: Loads immediate byte b into register vX
            0x6 => self.load_immediate(x, b),
            // 7xbb: Adds immediate byte b to register vX
            0x7 => self.add_immediate(x, b),
            // # Performs ALU operation
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | 8xy0 | Y = X                              |
            // | 8xy1 | X = X | Y                          |
            // | 8xy2 | X = X & Y                          |
            // | 8xy3 | X = X ^ Y                          |
            // | 8xy4 | X = X + Y; Set vF=carry            |
            // | 8xy5 | X = X - Y; Set vF=carry            |
            // | 8xy6 | X = X >> 1                         |
            // | 8xy7 | X = Y - X; Set vF=carry            |
            // | 8xyE | X = X << 1                         |
            0x8 => match n {
                0x0 => self.load(x, y),
                0x1 => self.or(x, y),
                0x2 => self.and(x, y),
                0x3 => self.xor(x, y),
                0x4 => self.add(x, y),
                0x5 => self.sub(x, y),
                0x6 => self.shift_right(x, y),
                0x7 => self.backwards_sub(x, y),
                0xE => self.shift_left(x, y),
                _ => self.unimplemented(opcode),
            },
            // # Performs a register-register comparison
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | 9XY0 | Skip next instruction if vX != vY  |
            0x9 => match n {
                0 => self.skip_not_equals(x, y),
                _ => self.unimplemented(opcode),
            },
            // Aaaa: Load address #a into register I
            0xa => self.load_i_immediate(a),
            // Baaa: Jump to &adr + v0
            0xb => self.jump_indexed(a),
            // Cxbb: Stores a random number + the provided byte into vX
            0xc => self.rand(x, b),
            // Dxyn: Draws n-byte sprite to the screen at coordinates (vX, vY)
            0xd => self.draw(x, y, n, bus),

            // # Skips instruction on value of keypress
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | eX9e | Skip next instruction if key == vX |
            // | eXa1 | Skip next instruction if key != vX |
            0xe => match b {
                0x9e => self.skip_key_equals(x),
                0xa1 => self.skip_key_not_equals(x),
                _ => self.unimplemented(opcode),
            },

            // # Performs IO
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | fX07 | Set vX to value in delay timer     |
            // | fX0a | Wait for input, store in vX m      |
            // | fX15 | Set sound timer to the value in vX |
            // | fX18 | set delay timer to the value in vX |
            // | fX1e | Add x to I                         |
            // | fX29 | Load sprite for character x into I |
            // | fX33 | BCD convert X into I[0..3]         |
            // | fX55 | DMA Stor from I to registers 0..X  |
            // | fX65 | DMA Load from I to registers 0..X  |
            0xf => match b {
                0x07 => self.load_delay_timer(x),
                0x0A => self.wait_for_key(x),
                0x15 => self.store_delay_timer(x),
                0x18 => self.store_sound_timer(x),
                0x1E => self.add_i(x),
                0x29 => self.load_sprite(x),
                0x33 => self.bcd_convert(x, bus),
                0x55 => self.store_dma(x, bus),
                0x65 => self.load_dma(x, bus),
                _ => self.unimplemented(opcode),
            },
            _ => unreachable!("Extracted nibble from byte, got >nibble?"),
        }
        let elapsed = self.timer.elapsed();
        // Print opcode disassembly:
        if self.flags.debug {
            std::println!(
                "{:3} {:03x}: {:<36}{:?}",
                self.cycle.bright_black(),
                pc,
                self.disassembler.instruction(opcode),
                elapsed.dimmed()
            );
        }
        // process breakpoints
        if self.breakpoints.contains(&self.pc) {
            self.flags.pause = true;
        }
        self
    }

    /// Dumps the current state of all CPU registers, and the cycle count
    /// # Examples
    /// ```rust
    ///# use chirp::prelude::*;
    ///# fn main() -> Result<()> {
    ///     let mut cpu = CPU::default();
    ///     cpu.dump();
    ///#    Ok(())
    ///# }
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
            self.sp,
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

impl Default for CPU {
    /// Constructs a new CPU with sane defaults and debug mode ON
    ///
    /// | value  | default | description
    /// |--------|---------|------------
    /// | screen |`0x0f00` | Location of screen memory.
    /// | font   |`0x0050` | Location of font memory.
    /// | pc     |`0x0200` | Start location. Generally 0x200 or 0x600.
    /// | sp     |`0x0efe` | Initial top of stack.
    ///
    ///
    /// # Examples
    /// ```rust
    /// use chirp::prelude::*;
    /// let mut cpu = CPU::default();
    /// ```
    fn default() -> Self {
        CPU {
            screen: 0xf00,
            font: 0x050,
            pc: 0x200,
            sp: 0xefe,
            i: 0,
            v: [0; 16],
            delay: 0.0,
            sound: 0.0,
            cycle: 0,
            keys: [false; 16],
            flags: ControlFlags {
                debug: true,
                ..Default::default()
            },
            timer: Instant::now(),
            breakpoints: vec![],
            disassembler: Disassemble::default(),
        }
    }
}

// Below this point, comments may be duplicated per impl' block,
// since some opcodes handle multiple instructions.

// | 0aaa | Issues a "System call" (ML routine)
//
// |opcode| effect                             |
// |------|------------------------------------|
// | 00e0 | Clear screen memory to all 0       |
// | 00ee | Return from subroutine             |
impl CPU {
    /// Unused instructions
    #[inline]
    fn unimplemented(&self, opcode: u16) {
        unimplemented!("Opcode: {opcode:04x}")
    }
    /// 0aaa: Handles a "machine language function call" (lmao)
    #[inline]
    fn sys(&mut self, a: Adr) {
        unimplemented!("SYS\t{a:03x}");
    }
    /// 00e0: Clears the screen memory to 0
    #[inline]
    fn clear_screen(&mut self, bus: &mut Bus) {
        if let Some(screen) = bus.get_region_mut(Region::Screen) {
            for byte in screen {
                *byte = 0;
            }
        }
    }
    /// 00ee: Returns from subroutine
    #[inline]
    fn ret(&mut self, bus: &impl Read<u16>) {
        self.sp = self.sp.wrapping_add(2);
        self.pc = bus.read(self.sp);
    }
}

// | 1aaa | Sets pc to an absolute address
impl CPU {
    /// 1aaa: Sets the program counter to an absolute address
    #[inline]
    fn jump(&mut self, a: Adr) {
        // jump to self == halt
        if a.wrapping_add(2) == self.pc {
            self.flags.pause = true;
        }
        self.pc = a;
    }
}

// | 2aaa | Pushes pc onto the stack, then jumps to a
impl CPU {
    /// 2aaa: Pushes pc onto the stack, then jumps to a
    #[inline]
    fn call(&mut self, a: Adr, bus: &mut impl Write<u16>) {
        bus.write(self.sp, self.pc);
        self.sp = self.sp.wrapping_sub(2);
        self.pc = a;
    }
}

// | 3xbb | Skips next instruction if register X == b
impl CPU {
    /// 3xbb: Skips the next instruction if register X == b
    #[inline]
    fn skip_equals_immediate(&mut self, x: Reg, b: u8) {
        if self.v[x] == b {
            self.pc = self.pc.wrapping_add(2);
        }
    }
}

// | 4xbb | Skips next instruction if register X != b
impl CPU {
    /// 4xbb: Skips the next instruction if register X != b
    #[inline]
    fn skip_not_equals_immediate(&mut self, x: Reg, b: u8) {
        if self.v[x] != b {
            self.pc = self.pc.wrapping_add(2);
        }
    }
}

// | 5xyn | Performs a register-register comparison
//
// |opcode| effect                             |
// |------|------------------------------------|
// | 5XY0 | Skip next instruction if vX == vY  |
impl CPU {
    /// 5xy0: Skips the next instruction if register X != register Y
    #[inline]
    fn skip_equals(&mut self, x: Reg, y: Reg) {
        if self.v[x] == self.v[y] {
            self.pc = self.pc.wrapping_add(2);
        }
    }
}

// | 6xbb | Loads immediate byte b into register vX
impl CPU {
    /// 6xbb: Loads immediate byte b into register vX
    #[inline]
    fn load_immediate(&mut self, x: Reg, b: u8) {
        self.v[x] = b;
    }
}

// | 7xbb | Adds immediate byte b to register vX
impl CPU {
    /// 7xbb: Adds immediate byte b to register vX
    #[inline]
    fn add_immediate(&mut self, x: Reg, b: u8) {
        self.v[x] = self.v[x].wrapping_add(b);
    }
}

// | 8xyn | Performs ALU operation
//
// |opcode| effect                             |
// |------|------------------------------------|
// | 8xy0 | Y = X                              |
// | 8xy1 | X = X | Y                          |
// | 8xy2 | X = X & Y                          |
// | 8xy3 | X = X ^ Y                          |
// | 8xy4 | X = X + Y; Set vF=carry            |
// | 8xy5 | X = X - Y; Set vF=carry            |
// | 8xy6 | X = X >> 1                         |
// | 8xy7 | X = Y - X; Set vF=carry            |
// | 8xyE | X = X << 1                         |
impl CPU {
    /// 8xy0: Loads the value of y into x
    #[inline]
    fn load(&mut self, x: Reg, y: Reg) {
        self.v[x] = self.v[y];
    }
    /// 8xy1: Performs bitwise or of vX and vY, and stores the result in vX
    ///
    /// # Quirk
    /// The original chip-8 interpreter will clobber vF for any 8-series instruction
    #[inline]
    fn or(&mut self, x: Reg, y: Reg) {
        self.v[x] |= self.v[y];
        if self.flags.quirks.bin_ops {
            self.v[0xf] = 0;
        }
    }
    /// 8xy2: Performs bitwise and of vX and vY, and stores the result in vX
    ///
    /// # Quirk
    /// The original chip-8 interpreter will clobber vF for any 8-series instruction
    #[inline]
    fn and(&mut self, x: Reg, y: Reg) {
        self.v[x] &= self.v[y];
        if self.flags.quirks.bin_ops {
            self.v[0xf] = 0;
        }
    }
    /// 8xy3: Performs bitwise xor of vX and vY, and stores the result in vX
    ///
    /// # Quirk
    /// The original chip-8 interpreter will clobber vF for any 8-series instruction
    #[inline]
    fn xor(&mut self, x: Reg, y: Reg) {
        self.v[x] ^= self.v[y];
        if self.flags.quirks.bin_ops {
            self.v[0xf] = 0;
        }
    }
    /// 8xy4: Performs addition of vX and vY, and stores the result in vX
    #[inline]
    fn add(&mut self, x: Reg, y: Reg) {
        let carry;
        (self.v[x], carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[0xf] = carry.into();
    }
    /// 8xy5: Performs subtraction of vX and vY, and stores the result in vX
    #[inline]
    fn sub(&mut self, x: Reg, y: Reg) {
        let carry;
        (self.v[x], carry) = self.v[x].overflowing_sub(self.v[y]);
        self.v[0xf] = (!carry).into();
    }
    /// 8xy6: Performs bitwise right shift of vX
    ///
    /// # Quirk
    /// On the original chip-8 interpreter, this shifts vY and stores the result in vX
    #[inline]
    fn shift_right(&mut self, x: Reg, y: Reg) {
        let src: Reg = if self.flags.quirks.shift { y } else { x };
        let shift_out = self.v[src] & 1;
        self.v[x] = self.v[src] >> 1;
        self.v[0xf] = shift_out;
    }
    /// 8xy7: Performs subtraction of vY and vX, and stores the result in vX
    #[inline]
    fn backwards_sub(&mut self, x: Reg, y: Reg) {
        let carry;
        (self.v[x], carry) = self.v[y].overflowing_sub(self.v[x]);
        self.v[0xf] = (!carry).into();
    }
    /// 8X_E: Performs bitwise left shift of vX
    ///
    /// # Quirk
    /// On the original chip-8 interpreter, this would perform the operation on vY
    /// and store the result in vX. This behavior was left out, for now.
    #[inline]
    fn shift_left(&mut self, x: Reg, y: Reg) {
        let src: Reg = if self.flags.quirks.shift { y } else { x };
        let shift_out: u8 = self.v[src] >> 7;
        self.v[x] = self.v[src] << 1;
        self.v[0xf] = shift_out;
    }
}

// | 9xyn | Performs a register-register comparison
//
// |opcode| effect                             |
// |------|------------------------------------|
// | 9XY0 | Skip next instruction if vX != vY  |
impl CPU {
    /// 9xy0: Skip next instruction if X != y
    #[inline]
    fn skip_not_equals(&mut self, x: Reg, y: Reg) {
        if self.v[x] != self.v[y] {
            self.pc = self.pc.wrapping_add(2);
        }
    }
}

// | Aaaa | Load address #a into register I
impl CPU {
    /// Aadr: Load address #adr into register I
    #[inline]
    fn load_i_immediate(&mut self, a: Adr) {
        self.i = a;
    }
}

// | Baaa | Jump to &adr + v0
impl CPU {
    /// Badr: Jump to &adr + v0
    ///
    /// Quirk:
    /// On the Super-Chip, this does stupid shit
    #[inline]
    fn jump_indexed(&mut self, a: Adr) {
        let reg = if self.flags.quirks.stupid_jumps {
            a as usize >> 8
        } else {
            0
        };
        self.pc = a.wrapping_add(self.v[reg] as Adr);
    }
}

// | Cxbb | Stores a random number + the provided byte into vX
impl CPU {
    /// Cxbb: Stores a random number & the provided byte into vX
    #[inline]
    fn rand(&mut self, x: Reg, b: u8) {
        self.v[x] = random::<u8>() & b;
    }
}

// | Dxyn | Draws n-byte sprite to the screen at coordinates (vX, vY)
impl CPU {
    /// Dxyn: Draws n-byte sprite to the screen at coordinates (vX, vY)
    ///
    /// # Quirk
    /// On the original chip-8 interpreter, this will wait for a VBI
    fn draw(&mut self, x: Reg, y: Reg, n: Nib, bus: &mut Bus) {
        let (x, y) = (self.v[x] as u16 % 64, self.v[y] as u16 % 32);
        if self.flags.quirks.draw_wait {
            self.flags.vbi_wait = true;
        }
        self.v[0xf] = 0;
        for byte in 0..n as u16 {
            if y + byte > 32 {
                return;
            }
            // Calculate the lower bound address based on the X,Y position on the screen
            let addr = (y + byte) * 8 + (x & 0x3f) / 8 + self.screen;
            // Read a byte of sprite data into a u16, and shift it x % 8 bits
            let sprite: u8 = bus.read(self.i + byte);
            let sprite = (sprite as u16) << 8 - (x & 7) & if x % 64 > 56 { 0xff00 } else { 0xffff };
            // Read a u16 from the bus containing the two bytes which might need to be updated
            let mut screen: u16 = bus.read(addr);
            // Save the bits-toggled-off flag if necessary
            if screen & sprite != 0 {
                self.v[0xF] = 1
            }
            // Update the screen word by XORing the sprite byte
            screen ^= sprite;
            // Save the result to the screen
            bus.write(addr, screen);
        }
    }
}

// | Exbb | Skips instruction on value of keypress
//
// |opcode| effect                             |
// |------|------------------------------------|
// | eX9e | Skip next instruction if key == #X |
// | eXa1 | Skip next instruction if key != #X |
impl CPU {
    /// Ex9E: Skip next instruction if key == #X
    #[inline]
    fn skip_key_equals(&mut self, x: Reg) {
        let x = self.v[x] as usize;
        if self.keys[x] {
            self.pc += 2;
        }
    }
    /// ExaE: Skip next instruction if key != #X
    #[inline]
    fn skip_key_not_equals(&mut self, x: Reg) {
        let x = self.v[x] as usize;
        if !self.keys[x] {
            self.pc += 2;
        }
    }
}

// | Fxbb | Performs IO
//
// |opcode| effect                             |
// |------|------------------------------------|
// | fX07 | Set vX to value in delay timer     |
// | fX0a | Wait for input, store in vX m      |
// | fX15 | Set sound timer to the value in vX |
// | fX18 | set delay timer to the value in vX |
// | fX1e | Add x to I                         |
// | fX29 | Load sprite for character x into I |
// | fX33 | BCD convert X into I[0..3]         |
// | fX55 | DMA Stor from I to registers 0..X  |
// | fX65 | DMA Load from I to registers 0..X  |
impl CPU {
    /// Fx07: Get the current DT, and put it in vX
    /// ```py
    /// vX = DT
    /// ```
    #[inline]
    fn load_delay_timer(&mut self, x: Reg) {
        self.v[x] = self.delay as u8;
    }
    /// Fx0A: Wait for key, then vX = K
    #[inline]
    fn wait_for_key(&mut self, x: Reg) {
        if let Some(key) = self.flags.lastkey {
            self.v[x] = key as u8;
            self.flags.lastkey = None;
        } else {
            self.pc = self.pc.wrapping_sub(2);
            self.flags.keypause = true;
        }
    }
    /// Fx15: Load vX into DT
    /// ```py
    /// DT = vX
    /// ```
    #[inline]
    fn store_delay_timer(&mut self, x: Reg) {
        self.delay = self.v[x] as f64;
    }
    /// Fx18: Load vX into ST
    /// ```py
    /// ST = vX;
    /// ```
    #[inline]
    fn store_sound_timer(&mut self, x: Reg) {
        self.sound = self.v[x] as f64;
    }
    /// Fx1e: Add vX to I,
    /// ```py
    /// I += vX;
    /// ```
    #[inline]
    fn add_i(&mut self, x: Reg) {
        self.i += self.v[x] as u16;
    }
    /// Fx29: Load sprite for character x into I
    /// ```py
    /// I = sprite(X);
    /// ```
    #[inline]
    fn load_sprite(&mut self, x: Reg) {
        self.i = self.font + (5 * (self.v[x] as Adr % 0x10));
    }
    /// Fx33: BCD convert X into I`[0..3]`
    #[inline]
    fn bcd_convert(&mut self, x: Reg, bus: &mut Bus) {
        let x = self.v[x];
        bus.write(self.i.wrapping_add(2), x % 10);
        bus.write(self.i.wrapping_add(1), x / 10 % 10);
        bus.write(self.i, x / 100 % 10);
    }
    /// Fx55: DMA Stor from I to registers 0..X
    ///
    /// # Quirk
    /// The original chip-8 interpreter uses I to directly index memory,
    /// with the side effect of leaving I as I+X+1 after the transfer is done.
    #[inline]
    fn store_dma(&mut self, x: Reg, bus: &mut Bus) {
        let i = self.i as usize;
        for (reg, value) in bus
            .get_mut(i..=i + x)
            .unwrap_or_default()
            .iter_mut()
            .enumerate()
        {
            *value = self.v[reg]
        }
        if self.flags.quirks.dma_inc {
            self.i += x as Adr + 1;
        }
    }
    /// Fx65: DMA Load from I to registers 0..X
    ///
    /// # Quirk
    /// The original chip-8 interpreter uses I to directly index memory,
    /// with the side effect of leaving I as I+X+1 after the transfer is done.
    #[inline]
    fn load_dma(&mut self, x: Reg, bus: &mut Bus) {
        let i = self.i as usize;
        for (reg, value) in bus
            .get(i + 0..=i + x)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            self.v[reg] = *value;
        }
        if self.flags.quirks.dma_inc {
            self.i += x as Adr + 1;
        }
    }
}
