// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE.txt for details)

//! A disassembler for Chip-8 opcodes

use super::{Adr, Nib, Reg};
use owo_colors::{OwoColorize, Style};
type Ins = Nib;

/// Extracts the I nibble of an IXYN instruction
#[inline]
pub fn i(ins: u16) -> Ins {
    (ins >> 12 & 0xf) as Ins
}
/// Extracts the X nibble of an IXYN instruction
#[inline]
pub fn x(ins: u16) -> Reg {
    (ins >> 8 & 0xf) as Reg
}
/// Extracts the Y nibble of an IXYN instruction
#[inline]
pub fn y(ins: u16) -> Reg {
    (ins >> 4 & 0xf) as Reg
}
/// Extracts the N nibble of an IXYN instruction
#[inline]
pub fn n(ins: u16) -> Nib {
    (ins & 0xf) as Nib
}
/// Extracts the B byte of an IXBB instruction
#[inline]
pub fn b(ins: u16) -> u8 {
    (ins & 0xff) as u8
}
/// Extracts the ADR trinibble of an IADR instruction
#[inline]
pub fn a(ins: u16) -> Adr {
    ins & 0x0fff
}

/// Disassembles Chip-8 instructions, printing them in the provided [owo_colors::Style]s
#[derive(Clone, Debug, PartialEq)]
pub struct Disassemble {
    invalid: Style,
    normal: Style,
}

impl Default for Disassemble {
    fn default() -> Self {
        Disassemble::builder().build()
    }
}

// Public API
impl Disassemble {
    /// Returns a new Disassemble with the provided Styles
    pub fn new(invalid: Style, normal: Style) -> Disassemble {
        Disassemble { invalid, normal }
    }
    /// Creates a [DisassembleBuilder], for partial configuration
    pub fn builder() -> DisassembleBuilder {
        DisassembleBuilder::default()
    }
    /// Disassemble a single instruction
    pub fn instruction(&self, opcode: u16) -> String {
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
                0x0e0 => self.clear_screen(),
                0x0ee => self.ret(),
                _ => self.sys(a),
            },
            // | 1aaa | Sets pc to an absolute address
            0x1 => self.jump(a),
            // | 2aaa | Pushes pc onto the stack, then jumps to a
            0x2 => self.call(a),
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
            0xd => self.draw(x, y, n),

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
                0x33 => self.bcd_convert_i(x),
                0x55 => self.dma_store(x),
                0x65 => self.dma_load(x),
                _ => self.unimplemented(opcode),
            },
            _ => unreachable!("Extracted nibble from byte, got >nibble?"),
        }
    }
}

// Private api
impl Disassemble {
    /// Unused instructions
    fn unimplemented(&self, opcode: u16) -> String {
        format!("inval  {opcode:04x}")
            .style(self.invalid)
            .to_string()
    }
    /// `0aaa`: Handles a "machine language function call" (lmao)
    pub fn sys(&self, a: Adr) -> String {
        format!("sysc   {a:03x}").style(self.invalid).to_string()
    }
    /// `00e0`: Clears the screen memory to 0
    pub fn clear_screen(&self) -> String {
        "cls    ".style(self.normal).to_string()
    }
    /// `00ee`: Returns from subroutine
    pub fn ret(&self) -> String {
        "ret    ".style(self.normal).to_string()
    }
    /// `1aaa`: Sets the program counter to an absolute address
    pub fn jump(&self, a: Adr) -> String {
        format!("jmp    {a:03x}").style(self.normal).to_string()
    }
    /// `2aaa`: Pushes pc onto the stack, then jumps to a
    pub fn call(&self, a: Adr) -> String {
        format!("call   {a:03x}").style(self.normal).to_string()
    }
    /// `3xbb`: Skips the next instruction if register X == b
    pub fn skip_if_x_equal_byte(&self, x: Reg, b: u8) -> String {
        format!("se     #{b:02x}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// `4xbb`: Skips the next instruction if register X != b
    pub fn skip_if_x_not_equal_byte(&self, x: Reg, b: u8) -> String {
        format!("sne    #{b:02x}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// `5xy0`: Skips the next instruction if register X != register Y
    pub fn skip_if_x_equal_y(&self, x: Reg, y: Reg) -> String {
        format!("se     v{x:X}, v{y:X}")
            .style(self.normal)
            .to_string()
    }

    /// `6xbb`: Loads immediate byte b into register vX
    pub fn load_immediate(&self, x: Reg, b: u8) -> String {
        format!("mov    #{b:02x}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// `7xbb`: Adds immediate byte b to register vX
    pub fn add_immediate(&self, x: Reg, b: u8) -> String {
        format!("add    #{b:02x}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// `8xy0`: Loads the value of y into x
    pub fn load_y_into_x(&self, x: Reg, y: Reg) -> String {
        format!("mov    v{y:X}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// `8xy1`: Performs bitwise or of vX and vY, and stores the result in vX
    pub fn x_orequals_y(&self, x: Reg, y: Reg) -> String {
        format!("or     v{y:X}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// `8xy2`: Performs bitwise and of vX and vY, and stores the result in vX
    pub fn x_andequals_y(&self, x: Reg, y: Reg) -> String {
        format!("and    v{y:X}, v{x:X}")
            .style(self.normal)
            .to_string()
    }

    /// `8xy3`: Performs bitwise xor of vX and vY, and stores the result in vX
    pub fn x_xorequals_y(&self, x: Reg, y: Reg) -> String {
        format!("xor    v{y:X}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// `8xy4`: Performs addition of vX and vY, and stores the result in vX
    pub fn x_addequals_y(&self, x: Reg, y: Reg) -> String {
        format!("add    v{y:X}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// `8xy5`: Performs subtraction of vX and vY, and stores the result in vX
    pub fn x_subequals_y(&self, x: Reg, y: Reg) -> String {
        format!("sub    v{y:X}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// `8xy6`: Performs bitwise right shift of vX
    pub fn shift_right_x(&self, x: Reg) -> String {
        format!("shr    v{x:X}").style(self.normal).to_string()
    }
    /// `8xy7`: Performs subtraction of vY and vX, and stores the result in vX
    pub fn backwards_subtract(&self, x: Reg, y: Reg) -> String {
        format!("bsub   v{y:X}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// 8X_E: Performs bitwise left shift of vX
    pub fn shift_left_x(&self, x: Reg) -> String {
        format!("shl    v{x:X}").style(self.normal).to_string()
    }
    /// `9xy0`: Skip next instruction if X != y
    pub fn skip_if_x_not_equal_y(&self, x: Reg, y: Reg) -> String {
        format!("sne    v{x:X}, v{y:X}")
            .style(self.normal)
            .to_string()
    }
    /// Aadr: Load address #adr into register I
    pub fn load_indirect_register(&self, a: Adr) -> String {
        format!("mov    ${a:03x}, I").style(self.normal).to_string()
    }
    /// Badr: Jump to &adr + v0
    pub fn jump_indexed(&self, a: Adr) -> String {
        format!("jmp    ${a:03x}+v0").style(self.normal).to_string()
    }
    /// `Cxbb`: Stores a random number + the provided byte into vX
    /// Pretty sure the input byte is supposed to be the seed of a LFSR or something
    pub fn rand(&self, x: Reg, b: u8) -> String {
        format!("rand   #{b:X}, v{x:X}")
            .style(self.normal)
            .to_string()
    }
    /// `Dxyn`: Draws n-byte sprite to the screen at coordinates (vX, vY)
    pub fn draw(&self, x: Reg, y: Reg, n: Nib) -> String {
        format!("draw   #{n:x}, v{x:X}, v{y:X}")
            .style(self.normal)
            .to_string()
    }
    /// `Ex9E`: Skip next instruction if key == #X
    pub fn skip_if_key_equals_x(&self, x: Reg) -> String {
        format!("sek    v{x:X}").style(self.normal).to_string()
    }
    /// `ExaE`: Skip next instruction if key != #X
    pub fn skip_if_key_not_x(&self, x: Reg) -> String {
        format!("snek   v{x:X}").style(self.normal).to_string()
    }
    /// `Fx07`: Get the current DT, and put it in vX
    /// ```py
    /// vX = DT
    /// ```
    pub fn get_delay_timer(&self, x: Reg) -> String {
        format!("mov    DT, v{x:X}").style(self.normal).to_string()
    }
    /// `Fx0A`: Wait for key, then vX = K
    pub fn wait_for_key(&self, x: Reg) -> String {
        format!("waitk  v{x:X}").style(self.normal).to_string()
    }
    /// `Fx15`: Load vX into DT
    /// ```py
    /// DT = vX
    /// ```
    pub fn load_delay_timer(&self, x: Reg) -> String {
        format!("mov    v{x:X}, DT").style(self.normal).to_string()
    }
    /// `Fx18`: Load vX into ST
    /// ```py
    /// ST = vX;
    /// ```
    pub fn load_sound_timer(&self, x: Reg) -> String {
        format!("mov    v{x:X}, ST").style(self.normal).to_string()
    }
    /// `Fx1e`: Add vX to I,
    /// ```py
    /// I += vX;
    /// ```
    pub fn add_to_indirect(&self, x: Reg) -> String {
        format!("add    v{x:X}, I").style(self.normal).to_string()
    }
    /// `Fx29`: Load sprite for character in vX into I
    /// ```py
    /// I = sprite(X);
    /// ```
    pub fn load_sprite_x(&self, x: Reg) -> String {
        format!("font   v{x:X}, I").style(self.normal).to_string()
    }
    /// `Fx33`: BCD convert X into I`[0..3]`
    pub fn bcd_convert_i(&self, x: Reg) -> String {
        format!("bcd    v{x:X}, &I").style(self.normal).to_string()
    }
    /// `Fx55`: DMA Stor from I to registers 0..X
    pub fn dma_store(&self, x: Reg) -> String {
        format!("dmao   v{x:X}").style(self.normal).to_string()
    }
    /// `Fx65`: DMA Load from I to registers 0..X
    pub fn dma_load(&self, x: Reg) -> String {
        format!("dmai   v{x:X}").style(self.normal).to_string()
    }
}

/// Builder for [Disassemble]rs
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisassembleBuilder {
    invalid: Option<Style>,
    normal: Option<Style>,
}

impl DisassembleBuilder {
    /// Styles invalid (or unimplemented) instructions
    pub fn invalid(mut self, style: Style) -> Self {
        self.invalid = Some(style);
        self
    }
    /// Styles valid (implemented) instructions
    pub fn normal(mut self, style: Style) -> Self {
        self.normal = Some(style);
        self
    }
    /// Builds a Disassemble
    pub fn build(self) -> Disassemble {
        Disassemble {
            invalid: if let Some(style) = self.invalid {
                style
            } else {
                Style::new().bold().red()
            },
            normal: if let Some(style) = self.normal {
                style
            } else {
                Style::new().green()
            },
        }
    }
}
