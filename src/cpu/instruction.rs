// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)
#![allow(clippy::bad_bit_mask)]
//! Contains the definition of a Chip-8 [Insn]

pub mod disassembler;

use imperative_rs::InstructionSet;
use std::fmt::Display;


#[allow(non_camel_case_types, non_snake_case, missing_docs)]
#[derive(Clone, Copy, Debug, InstructionSet, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Implements a Disassembler using imperative_rs
pub enum Insn {
    // Base instruction set
    /// | 00e0 | Clear screen memory to 0s
    #[opcode = "0x00e0"]
    cls,
    /// | 00ee | Return from subroutine
    #[opcode = "0x00ee"]
    ret,
    /// | 1aaa | Jumps to an absolute address
    #[opcode = "0x1AAA"]
    jmp { A: u16 },
    /// | 2aaa | Pushes pc onto the stack, then jumps to a
    #[opcode = "0x2AAA"]
    call { A: u16 },
    /// | 3xbb | Skips next instruction if register X == b
    #[opcode = "0x3xBB"]
    seb { B: u8, x: usize },
    /// | 4xbb | Skips next instruction if register X != b
    #[opcode = "0x4xBB"]
    sneb { B: u8, x: usize },
    /// | 9XY0 | Skip next instruction if vX == vY  |
    #[opcode = "0x5xy0"]
    se { y: usize, x: usize },
    /// | 6xbb | Loads immediate byte b into register vX
    #[opcode = "0x6xBB"]
    movb { B: u8, x: usize },
    /// | 7xbb | Adds immediate byte b to register vX
    #[opcode = "0x7xBB"]
    addb { B: u8, x: usize },
    /// | 8xy0 | Loads the value of y into x
    #[opcode = "0x8xy0"]
    mov { x: usize, y: usize },
    /// | 8xy1 | Performs bitwise or of vX and vY, and stores the result in vX
    #[opcode = "0x8xy1"]
    or { y: usize, x: usize },
    /// | 8xy2 | Performs bitwise and of vX and vY, and stores the result in vX
    #[opcode = "0x8xy2"]
    and { y: usize, x: usize },
    /// | 8xy3 | Performs bitwise xor of vX and vY, and stores the result in vX
    #[opcode = "0x8xy3"]
    xor { y: usize, x: usize },
    /// | 8xy4 | Performs addition of vX and vY, and stores the result in vX
    #[opcode = "0x8xy4"]
    add { y: usize, x: usize },
    /// | 8xy5 | Performs subtraction of vX and vY, and stores the result in vX
    #[opcode = "0x8xy5"]
    sub { y: usize, x: usize },
    /// | 8xy6 | Performs bitwise right shift of vX (or vY)
    #[opcode = "0x8xy6"]
    shr { y: usize, x: usize },
    /// | 8xy7 | Performs subtraction of vY and vX, and stores the result in vX
    #[opcode = "0x8xy7"]
    bsub { y: usize, x: usize },
    /// | 8xyE | Performs bitwise left shift of vX
    #[opcode = "0x8xye"]
    shl { y: usize, x: usize },
    /// | 9XY0 | Skip next instruction if vX != vY
    #[opcode = "0x9xy0"]
    sne { y: usize, x: usize },
    /// | Aaaa | Load address #a into register I
    #[opcode = "0xaAAA"]
    movI { A: u16 },
    /// | Baaa | Jump to &adr + v0
    #[opcode = "0xbAAA"]
    jmpr { A: u16 },
    /// | Cxbb | Stores a random number & the provided byte into vX
    #[opcode = "0xcxBB"]
    rand { B: u8, x: usize },
    /// | Dxyn | Draws n-byte sprite to the screen at coordinates (vX, vY)
    #[opcode = "0xdxyn"]
    draw { y: usize, x: usize, n: u8 },
    /// | eX9e | Skip next instruction if key == vX
    #[opcode = "0xex9e"]
    sek { x: usize },
    /// | eXa1 | Skip next instruction if key != vX
    #[opcode = "0xexa1"]
    snek { x: usize },
    /// | fX07 | Set vX to value in delay timer
    #[opcode = "0xfx07"]
    getdt { x: usize },
    /// | fX0a | Wait for input, store key in vX
    #[opcode = "0xfx0a"]
    waitk { x: usize },
    /// | fX15 | Set sound timer to the value in vX
    #[opcode = "0xfx15"]
    setdt { x: usize },
    /// | fX18 | set delay timer to the value in vX
    #[opcode = "0xfx18"]
    movst { x: usize },
    /// | fX1e | Add vX to I
    #[opcode = "0xfx1e"]
    addI { x: usize },
    /// | fX29 | Load sprite for character x into I
    #[opcode = "0xfx29"]
    font { x: usize },
    /// | fX33 | BCD convert X into I[0..3]
    #[opcode = "0xfx33"]
    bcd { x: usize },
    /// | fX55 | DMA Stor from I to registers 0..X
    #[opcode = "0xfx55"]
    dmao { x: usize },
    /// | fX65 | DMA Load from I to registers 0..X
    #[opcode = "0xfx65"]
    dmai { x: usize },

    // Super Chip extensions
    /// | 00cN | Scroll the screen down
    #[opcode = "0x00cn"]
    scd { n: u8 },
    /// | 00fb | Scroll the screen right
    #[opcode = "0x00fb"]
    scr,
    /// | 00fc | Scroll the screen left
    #[opcode = "0x00fc"]
    scl,
    /// | 00fd | Exit (halt and catch fire)
    #[opcode = "0x00fd"]
    halt,
    /// | 00fe | Return to low-resolution mode
    #[opcode = "0x00fe"]
    lores,
    /// | 00ff | Enter high-resolution mode
    #[opcode = "0x00ff"]
    hires,
    /// | fx30 | Enter high-resolution mode
    #[opcode = "0xfx30"]
    hfont { x: usize },
    /// | fx75 | Save to "flag registers"
    #[opcode = "0xfx75"]
    flgo { x: usize },
    /// | fx85 | Load from "flag registers"
    #[opcode = "0xfx85"]
    flgi { x: usize },

    // XO-Chip instructions
    /// | 00dN | Scroll the screen up
    #[opcode = "0x00dn"]
    scu { n: u8 },
    /// | 5XY2 | DMA Load from I to vX..vY
    #[opcode = "0x5xy2"]
    dmaro { y: usize, x: usize },
    /// | 5XY3 | DMA Load from I to vX..vY
    #[opcode = "0x5xy3"]
    dmari { y: usize, x: usize },
    /// | F000 | Load long address into character I
    #[opcode = "0xf000_iiii"]
    long { i: usize },
}

impl Display for Insn {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Base instruction set
            Insn::cls               => write!(f, "cls    "),
            Insn::ret               => write!(f, "ret    "),
            Insn::jmp { A }         => write!(f, "jmp    {A:03x}"),
            Insn::call { A }        => write!(f, "call   {A:03x}"),
            Insn::seb { B, x }      => write!(f, "se     #{B:02x}, v{x:X}"),
            Insn::sneb { B, x }     => write!(f, "sne    #{B:02x}, v{x:X}"),
            Insn::se { y, x }       => write!(f, "se     v{y:X}, v{x:X}"),
            Insn::movb { B, x }     => write!(f, "mov    #{B:02x}, v{x:X}"),
            Insn::addb { B, x }     => write!(f, "add    #{B:02x}, v{x:X}"),
            Insn::mov { x, y }      => write!(f, "mov    v{y:X}, v{x:X}"),
            Insn::or { y, x }       => write!(f, "or     v{y:X}, v{x:X}"),
            Insn::and { y, x }      => write!(f, "and    v{y:X}, v{x:X}"),
            Insn::xor { y, x }      => write!(f, "xor    v{y:X}, v{x:X}"),
            Insn::add { y, x }      => write!(f, "add    v{y:X}, v{x:X}"),
            Insn::sub { y, x }      => write!(f, "sub    v{y:X}, v{x:X}"),
            Insn::shr { y, x }      => write!(f, "shr    v{y:X}, v{x:X}"),
            Insn::bsub { y, x }     => write!(f, "bsub   v{y:X}, v{x:X}"),
            Insn::shl { y, x }      => write!(f, "shl    v{y:X}, v{x:X}"),
            Insn::sne { y, x }      => write!(f, "sne    v{y:X}, v{x:X}"),
            Insn::movI { A }        => write!(f, "mov    ${A:03x}, I"),
            Insn::jmpr { A }        => write!(f, "jmp    ${A:03x}+v0"),
            Insn::rand { B, x }     => write!(f, "rand   #{B:02x}, v{x:X}"),
            Insn::draw { y, x, n }  => write!(f, "draw   #{n:x}, v{x:X}, v{y:X}"),
            Insn::sek { x }         => write!(f, "sek    v{x:X}"),
            Insn::snek { x }        => write!(f, "snek   v{x:X}"),
            Insn::getdt { x }       => write!(f, "mov    DT, v{x:X}"),
            Insn::waitk { x }       => write!(f, "waitk  v{x:X}"),
            Insn::setdt { x }       => write!(f, "mov    v{x:X}, DT"),
            Insn::movst { x }       => write!(f, "mov    v{x:X}, ST"),
            Insn::addI { x }        => write!(f, "add    v{x:X}, I"),
            Insn::font { x }        => write!(f, "font   v{x:X}, I"),
            Insn::bcd { x }         => write!(f, "bcd    v{x:X}, &I"),
            Insn::dmao { x }        => write!(f, "dmao   v{x:X}"),
            Insn::dmai { x }        => write!(f, "dmai   v{x:X}"),
            // Super Chip extensions
            Insn::scd { n }         => write!(f, "scd    #{n:x}"),
            Insn::scr               => write!(f, "scr    "),
            Insn::scl               => write!(f, "scl    "),
            Insn::halt              => write!(f, "halt   "),
            Insn::lores             => write!(f, "lores  "),
            Insn::hires             => write!(f, "hires  "),
            Insn::hfont { x }       => write!(f, "hfont  v{x:X}"),
            Insn::flgo { x }        => write!(f, "flgo   v{x:X}"),
            Insn::flgi { x }        => write!(f, "flgi   v{x:X}"),
            // XO-Chip extensions
            Insn::scu { n }         => write!(f, "scu    #{n:x}"),
            Insn::dmaro { y, x }    => write!(f, "dmaro  v{x:X}..v{y:X}"),
            Insn::dmari { y, x }    => write!(f, "dmari  v{x:X}..v{y:X}"),
            Insn::long { i }        => write!(f, "long   ${i:04x}"),
        }  
    }
}
