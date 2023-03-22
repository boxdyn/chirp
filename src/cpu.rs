//! The CPU decodes and runs instructions

pub mod disassemble;

use self::disassemble::Disassemble;
use crate::bus::{Bus, Read, Write};
use owo_colors::OwoColorize;
use rand::random;
use std::time::Instant;

type Reg = usize;
type Adr = u16;
type Nib = u8;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlFlags {
    pub debug: bool,
    pub pause: bool,
    pub keypause: bool,
    pub authentic: bool,
}

impl ControlFlags {
    pub fn debug(&mut self) {
        self.debug = !self.debug
    }
    pub fn pause(&mut self) {
        self.pause = !self.pause
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Keys {
    keys: [bool; 16],
}

#[derive(Clone, Debug, PartialEq)]
pub struct CPU {
    // memory map info
    screen: Adr,
    font: Adr,
    // registers
    pc: Adr,
    sp: Adr,
    i: Adr,
    v: [u8; 16],
    delay: u8,
    sound: u8,
    // I/O
    pub keys: [bool; 16],
    pub flags: ControlFlags,
    // Execution data
    cycle: usize,
    breakpoints: Vec<Adr>,
    disassembler: Disassemble,
}

// public interface
impl CPU {
    /// Press keys (where `keys` is a bitmap of the keys [F-0])
    pub fn press(&mut self, key: usize) {
        if (0..16).contains(&key) {
            self.keys[key] = true;
            self.flags.keypause = false;
        }
    }
    /// Release all keys
    pub fn release(&mut self) {
        for key in &mut self.keys {
            *key = false;
        }
    }

    /// Set a general purpose register in the CPU
    /// # Examples
    /// ```rust
    /// # use chumpulator::prelude::*;
    /// // Create a new CPU, and set v4 to 0x41
    /// let cpu = CPU::default()
    ///     .set_gpr(0x4, 0x41);
    /// // Dump the CPU registers
    /// cpu.dump();
    /// ```
    pub fn set_gpr(&mut self, gpr: Reg, value: u8) {
        if let Some(gpr) = self.v.get_mut(gpr) {
            *gpr = value;
        }
    }

    /// Constructs a new CPU with sane defaults
    ///
    /// | value  | default | description
    /// |--------|---------|------------
    /// | screen | 0x0f00  | Location of screen memory.
    /// | font   | 0x0050  | Location of font memory.
    /// | pc     | 0x0200  | Start location. Generally 0x200 or 0x600.
    /// | sp     | 0x0efe  | Initial top of stack.
    /// # Examples
    /// ```rust
    /// # use chumpulator::prelude::*;
    /// let mut cpu = CPU::new(0xf00, 0x50, 0x200, 0xefe, Disassemble::default());
    /// ```
    pub fn new(screen: Adr, font: Adr, pc: Adr, sp: Adr, disassembler: Disassemble) -> Self {
        CPU {
            disassembler,
            screen,
            font,
            pc,
            sp,
            i: 0,
            v: [0; 16],
            delay: 0,
            sound: 0,
            cycle: 0,
            keys: [false; 16],
            breakpoints: vec![],
            flags: ControlFlags {
                debug: true,
                ..Default::default()
            },
        }
    }

    /// Get the program counter
    pub fn pc(&self) -> Adr {
        self.pc
    }

    /// Soft resets the CPU, releasing keypause and reinitializing the program counter to 0x200
    pub fn soft_reset(&mut self) {
        self.pc = 0x200;
        self.flags.keypause = false;
    }

    /// Set a breakpoint
    pub fn set_break(&mut self, point: Adr) {
        if !self.breakpoints.contains(&point) {
            self.breakpoints.push(point)
        }
    }

    /// Unset a breakpoint
    pub fn unset_break(&mut self, point: Adr) {
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
    }

    /// Unpauses the emulator for a single tick
    /// NOTE: does not synchronize with delay timers
    pub fn singlestep(&mut self, bus: &mut Bus) {
            self.flags.pause = false;
            self.tick(bus);
            self.flags.pause = true;
    }

    /// Ticks the delay and sound timers
    pub fn tick_timer(&mut self) {
        if self.flags.pause {
            return;
        }
        self.delay = self.delay.saturating_sub(1);
        self.sound = self.sound.saturating_sub(1);
    }

    /// Runs a single instruction
    pub fn tick(&mut self, bus: &mut Bus) {
        // Do nothing if paused
        if self.flags.pause || self.flags.keypause {
            return;
        }
        let time = Instant::now();
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
            0x3 => self.skip_if_x_equal_byte(x, b),
            // | 4xbb | Skips next instruction if register X != b
            0x4 => self.skip_if_x_not_equal_byte(x, b),
            // # Performs a register-register comparison
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | 9XY0 | Skip next instruction if vX == vY  |
            0x5 => match n {
                0x0 => self.skip_if_x_equal_y(x, y),
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
                0x0 => self.load_y_into_x(x, y),
                0x1 => self.x_orequals_y(x, y),
                0x2 => self.x_andequals_y(x, y),
                0x3 => self.x_xorequals_y(x, y),
                0x4 => self.x_addequals_y(x, y),
                0x5 => self.x_subequals_y(x, y),
                0x6 => self.shift_right_x(x),
                0x7 => self.backwards_subtract(x, y),
                0xE => self.shift_left_x(x),
                _ => self.unimplemented(opcode),
            },
            // # Performs a register-register comparison
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | 9XY0 | Skip next instruction if vX != vY  |
            0x9 => match n {
                0 => self.skip_if_x_not_equal_y(x, y),
                _ => self.unimplemented(opcode),
            },
            // Aaaa: Load address #a into register I
            0xa => self.load_indirect_register(a),
            // Baaa: Jump to &adr + v0
            0xb => self.jump_indexed(a),
            // Cxbb: Stores a random number + the provided byte into vX
            0xc => self.rand(x, b),
            // Dxyn: Draws n-byte sprite to the screen at coordinates (vX, vY)
            0xd => self.draw(x, y, n, bus),

            // # Skips instruction on value of keypress
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | eX9e | Skip next instruction if key == #X |
            // | eXa1 | Skip next instruction if key != #X |
            0xe => match b {
                0x9e => self.skip_if_key_equals_x(x),
                0xa1 => self.skip_if_key_not_x(x),
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
                0x07 => self.get_delay_timer(x),
                0x0A => self.wait_for_key(x),
                0x15 => self.load_delay_timer(x),
                0x18 => self.load_sound_timer(x),
                0x1E => self.add_to_indirect(x),
                0x29 => self.load_sprite_x(x),
                0x33 => self.bcd_convert_i(x, bus),
                0x55 => self.dma_store(x, bus),
                0x65 => self.dma_load(x, bus),
                _ => self.unimplemented(opcode),
            },
            _ => unimplemented!("Extracted nibble from byte, got >nibble?"),
            
        }
        let elapsed = time.elapsed();
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
        self.cycle += 1;
        // process breakpoints
        if self.breakpoints.contains(&self.pc) {
            self.flags.pause = true;
        }
    }

    pub fn dump(&self) {
        let dumpstyle = owo_colors::Style::new().bright_black();
        let mut dump = format!(
            "PC: {:04x}, SP: {:04x}, I: {:04x}\n",
            self.pc, self.sp, self.i
        );
        for (i, gpr) in self.v.into_iter().enumerate() {
            dump += &format!(
                "V{i:x}: {:02x} {}",
                gpr,
                match i % 4 {
                    3 => "\n",
                    _ => "",
                }
            )
        }
        dump += &format!("DLY: {}, SND: {}", self.delay, self.sound);

        std::println!("{}", dump.style(dumpstyle));
    }
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            screen: 0xf00,
            font: 0x050,
            pc: 0x200,
            sp: 0xefe,
            i: 0,
            v: [0; 16],
            delay: 0,
            sound: 0,
            cycle: 0,
            keys: [false; 16],
            flags: ControlFlags {
                debug: true,
                ..Default::default()
            },
            breakpoints: vec![],
            disassembler: Disassemble::default(),
        }
    }
}

// private implementation
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
        if let Some(screen) = bus.get_region_mut("screen") {
            for byte in screen {
                *byte = 0;
            }
        }
        //use dump::BinDumpable;
        //bus.bin_dump(self.screen as usize..self.screen as usize + 0x100);
    }
    /// 00ee: Returns from subroutine
    #[inline]
    fn ret(&mut self, bus: &impl Read<u16>) {
        self.sp = self.sp.wrapping_add(2);
        self.pc = bus.read(self.sp);
    }
    /// 1aaa: Sets the program counter to an absolute address
    #[inline]
    fn jump(&mut self, a: Adr) {
        self.pc = a;
    }
    /// 2aaa: Pushes pc onto the stack, then jumps to a
    #[inline]
    fn call(&mut self, a: Adr, bus: &mut impl Write<u16>) {
        bus.write(self.sp, self.pc);
        self.sp = self.sp.wrapping_sub(2);
        self.pc = a;
    }
    /// 3xbb: Skips the next instruction if register X == b
    #[inline]
    fn skip_if_x_equal_byte(&mut self, x: Reg, b: u8) {
        if self.v[x] == b {
            self.pc = self.pc.wrapping_add(2);
        }
    }
    /// 4xbb: Skips the next instruction if register X != b
    #[inline]
    fn skip_if_x_not_equal_byte(&mut self, x: Reg, b: u8) {
        if self.v[x] != b {
            self.pc = self.pc.wrapping_add(2);
        }
    }
    /// 5xy0: Skips the next instruction if register X != register Y
    #[inline]
    fn skip_if_x_equal_y(&mut self, x: Reg, y: Reg) {
        if self.v[x] == self.v[y] {
            self.pc = self.pc.wrapping_add(2);
        }
    }
    /// 6xbb: Loads immediate byte b into register vX
    #[inline]
    fn load_immediate(&mut self, x: Reg, b: u8) {
        self.v[x] = b;
    }
    /// 7xbb: Adds immediate byte b to register vX
    #[inline]
    fn add_immediate(&mut self, x: Reg, b: u8) {
        self.v[x] = self.v[x].wrapping_add(b);
    }
    /// Set the carry register (vF) after math
    #[inline]
    fn set_carry(&mut self, x: Reg, y: Reg, f: fn(u16, u16) -> u16, inv: bool) -> u8 {
        let sum = f(self.v[x] as u16, self.v[y] as u16);
        self.v[0xf] = if (sum & 0xff00 != 0) ^ inv { 1 } else { 0 };
        (sum & 0xff) as u8
    }
    /// 8xy0: Loads the value of y into x
    #[inline]
    fn load_y_into_x(&mut self, x: Reg, y: Reg) {
        self.v[x] = self.v[y];
    }
    /// 8xy1: Performs bitwise or of vX and vY, and stores the result in vX
    #[inline]
    fn x_orequals_y(&mut self, x: Reg, y: Reg) {
        self.v[x] |= self.v[y];
    }
    /// 8xy2: Performs bitwise and of vX and vY, and stores the result in vX
    #[inline]
    fn x_andequals_y(&mut self, x: Reg, y: Reg) {
        self.v[x] &= self.v[y];
    }
    /// 8xy3: Performs bitwise xor of vX and vY, and stores the result in vX
    #[inline]
    fn x_xorequals_y(&mut self, x: Reg, y: Reg) {
        self.v[x] ^= self.v[y];
    }
    /// 8xy4: Performs addition of vX and vY, and stores the result in vX
    #[inline]
    fn x_addequals_y(&mut self, x: Reg, y: Reg) {
        self.v[x] = self.set_carry(x, y, u16::wrapping_add, false);
    }
    /// 8xy5: Performs subtraction of vX and vY, and stores the result in vX
    #[inline]
    fn x_subequals_y(&mut self, x: Reg, y: Reg) {
        self.v[x] = self.set_carry(x, y, u16::wrapping_sub, true);
    }
    /// 8xy6: Performs bitwise right shift of vX
    #[inline]
    fn shift_right_x(&mut self, x: Reg) {
        let shift_out = self.v[x] & 1;
        self.v[x] >>= 1;
        self.v[0xf] = shift_out;
    }
    /// 8xy7: Performs subtraction of vY and vX, and stores the result in vX
    #[inline]
    fn backwards_subtract(&mut self, x: Reg, y: Reg) {
        self.v[x] = self.set_carry(y, x, u16::wrapping_sub, true);
    }
    /// 8X_E: Performs bitwise left shift of vX
    #[inline]
    fn shift_left_x(&mut self, x: Reg) {
        let shift_out: u8 = self.v[x] >> 7;
        self.v[x] <<= 1;
        self.v[0xf] = shift_out;
    }

    /// 9xy0: Skip next instruction if X != y
    #[inline]
    fn skip_if_x_not_equal_y(&mut self, x: Reg, y: Reg) {
        if self.v[x] != self.v[y] {
            self.pc = self.pc.wrapping_add(2);
        }
    }
    /// Aadr: Load address #adr into register I
    #[inline]
    fn load_indirect_register(&mut self, a: Adr) {
        self.i = a;
    }
    /// Badr: Jump to &adr + v0
    #[inline]
    fn jump_indexed(&mut self, a: Adr) {
        self.pc = a.wrapping_add(self.v[0] as Adr);
    }
    /// Cxbb: Stores a random number & the provided byte into vX
    #[inline]
    fn rand(&mut self, x: Reg, b: u8) {
        self.v[x] = random::<u8>() & b;
    }
    /// Dxyn: Draws n-byte sprite to the screen at coordinates (vX, vY)
    #[inline]
    fn draw(&mut self, x: Reg, y: Reg, n: Nib, bus: &mut Bus) {
        // println!("{}", format_args!("draw\t#{n:x}, (x: {:x}, y: {:x})", self.v[x], self.v[y]).green());
        let (x, y) = (self.v[x], self.v[y]);
        self.v[0xf] = 0;
        for byte in 0..n as u16 {
            // Calculate the lower bound address based on the X,Y position on the screen
            let addr = ((y as u16 + byte) * 8) + (x / 8) as u16 + self.screen;
            // Read a byte of sprite data into a u16, and shift it x % 8 bits
            let sprite: u8 = bus.read(self.i + byte);
            let sprite = (sprite as u16) << 1 + (7 - x % 8);
            // Read a u16 from the bus containing the two bytes which might need to be updated
            let mut screen: u16 = bus.read(addr);
            // Save the bits-toggled-off flag if necessary
            if screen & sprite != 0 {
                self.v[0xF] = 1
            }
            // Update the screen word by XORing the sprite byte
            screen ^= sprite;
            // Save the result back to the screen
            bus.write(addr, screen);
        }
    }
    /// Ex9E: Skip next instruction if key == #X
    #[inline]
    fn skip_if_key_equals_x(&mut self, x: Reg) {
        let x = self.v[x] as usize;
        if self.keys[x] {
            self.pc += 2;
        }
    }
    /// ExaE: Skip next instruction if key != #X
    #[inline]
    fn skip_if_key_not_x(&mut self, x: Reg) {
        let x = self.v[x] as usize;
        if !self.keys[x] {
            self.pc += 2;
        }
    }
    /// Fx07: Get the current DT, and put it in vX
    /// ```py
    /// vX = DT
    /// ```
    #[inline]
    fn get_delay_timer(&mut self, x: Reg) {
        self.v[x] = self.delay;
    }
    /// Fx0A: Wait for key, then vX = K
    #[inline]
    fn wait_for_key(&mut self, x: Reg) {
        let mut pressed = false;
        for bit in 0..16 {
            if self.keys[bit] {
                self.v[x] = bit as u8;
                pressed = true;
            }
        }
        if !pressed {
            self.pc = self.pc.wrapping_sub(2);
            self.flags.keypause = true;
        }
    }
    /// Fx15: Load vX into DT
    /// ```py
    /// DT = vX
    /// ```
    #[inline]
    fn load_delay_timer(&mut self, x: Reg) {
        self.delay = self.v[x];
    }
    /// Fx18: Load vX into ST
    /// ```py
    /// ST = vX;
    /// ```
    #[inline]
    fn load_sound_timer(&mut self, x: Reg) {
        self.sound = self.v[x];
    }
    /// Fx1e: Add vX to I,
    /// ```py
    /// I += vX;
    /// ```
    #[inline]
    fn add_to_indirect(&mut self, x: Reg) {
        self.i += self.v[x] as u16;
    }
    /// Fx29: Load sprite for character x into I
    /// ```py
    /// I = sprite(X);
    /// ```
    #[inline]
    fn load_sprite_x(&mut self, x: Reg) {
        self.i = self.font + (5 * (self.v[x] as Adr % 0x10));
    }
    /// Fx33: BCD convert X into I`[0..3]`
    #[inline]
    fn bcd_convert_i(&mut self, x: Reg, bus: &mut Bus) {
        let x = self.v[x];
        bus.write(self.i.wrapping_add(2), x % 10);
        bus.write(self.i.wrapping_add(1), x / 10 % 10);
        bus.write(self.i, x / 100 % 10);
    }
    /// Fx55: DMA Stor from I to registers 0..X
    #[inline]
    fn dma_store(&mut self, x: Reg, bus: &mut Bus) {
        let i = self.i as usize;
        for (reg, value) in bus
            .get_mut(i..=i + x)
            .unwrap_or_default()
            .iter_mut()
            .enumerate()
        {
            *value = self.v[reg]
        }
        if self.flags.authentic {
            self.i += x as Adr + 1;
        }
    }
    /// Fx65: DMA Load from I to registers 0..X
    #[inline]
    fn dma_load(&mut self, x: Reg, bus: &mut Bus) {
        let i = self.i as usize;
        for (reg, value) in bus
            .get(i + 0..=i + x)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            self.v[reg] = *value;
        }
        if self.flags.authentic {
            self.i += x as Adr + 1;
        }
    }
}
