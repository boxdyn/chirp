//! Represents a chip-8 instruction as a Rust enum

use super::{Adr, Nib, Reg};
type Word = Adr;
type Byte = u8;
type Ins = Nib;

/// Extract the instruction nibble from a word
#[inline]
pub fn i(ins: Word) -> Ins {
    (ins >> 12) as Ins & 0xf
}
/// Extracts the X-register nibble from a word
#[inline]
pub fn x(ins: Word) -> Reg {
    ins as Reg >> 8 & 0xf
}
/// Extracts the Y-register nibble from a word
#[inline]
pub fn y(ins: u16) -> Reg {
    ins as Reg >> 4 & 0xf
}
/// Extracts the nibble-sized immediate from a word
#[inline]
pub fn n(ins: Word) -> Nib {
    ins as Nib & 0xf
}
/// Extracts the byte-sized immediate from a word
#[inline]
pub fn b(ins: Word) -> Byte {
    ins as Byte
}
/// Extracts the address-sized immediate from a word
#[inline]
pub fn a(ins: Word) -> Adr {
    ins & 0x0fff
}
/// Restores the instruction nibble into a word
#[inline]
pub fn ii(i: Ins) -> u16 {
    (i as Word & 0xf) << 12
}
/// Restores the X-register nibble into a word
#[inline]
pub fn xi(x: Reg) -> Word {
    (x as Word & 0xf) << 8
}
/// Restores the Y-register nibble into a word
#[inline]
pub fn yi(y: Reg) -> Word {
    (y as Word & 0xf) << 4
}
/// Restores the nibble-sized immediate into a word
#[inline]
pub fn ni(n: Nib) -> Word {
    n as Word & 0xf
}
/// Restores the byte-sized immediate into a word
#[inline]
pub fn bi(b: Byte) -> Word {
    b as Word
}
/// Captures the operand and type of a Chip-8 instruction
pub enum Chip8Instruction {
    Unimplemented(Word),
    Clear,
    Return,
    Sys(Adr),
    Jump(Adr),
    Call(Adr),
    SkipEqualsByte(Reg, Byte),
    SkipNotEqualsByte(Reg, Byte),
    SkipEquals(Reg, Reg),
    LoadImmediate(Reg, Byte),
    AddImmediate(Reg, Byte),
    Copy(Reg, Reg),
    Or(Reg, Reg),
    And(Reg, Reg),
    Xor(Reg, Reg),
    Add(Reg, Reg),
    Sub(Reg, Reg),
    ShiftRight(Reg, Reg),
    BackwardsSub(Reg, Reg),
    ShiftLeft(Reg, Reg),
    SkipNotEquals(Reg, Reg),
    LoadIndirect(Adr),
    JumpIndexed(Adr),
    Rand(Reg, Byte),
    Draw(Reg, Reg, Nib),
    SkipEqualsKey(Reg),
    SkipNotEqualsKey(Reg),
    StoreDelay(Reg),
    WaitForKey(Reg),
    LoadDelay(Reg),
    LoadSound(Reg),
    AddIndirect(Reg),
    LoadSprite(Reg),
    BcdConvert(Reg),
    DmaStore(Reg),
    DmaLoad(Reg),
}

impl TryFrom<Word> for Chip8Instruction {
    type Error = crate::error::Error;
    /// Converts a 16-bit word into a Chip8Instruction, when possible.
    fn try_from(opcode: Word) -> Result<Self, Self::Error> {
        use crate::error::Error::*;
        let (i, x, y, n, b, a) = (
            i(opcode),
            x(opcode),
            y(opcode),
            n(opcode),
            b(opcode),
            a(opcode),
        );
        if i > 0xf {
            return Err(FunkyMath {
                word: opcode,
                explanation: "Instruction nibble greater than 0xf".into(),
            });
        }
        Ok(match i {
            // # Issue a system call
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | 00e0 | Clear screen memory to all 0       |
            // | 00ee | Return from subroutine             |
            0x0 => match a {
                0xe0 => Self::Clear,
                0xee => Self::Return,
                _ => Self::Sys(a),
            },
            // | 1aaa | Sets pc to an absolute address
            0x1 => Self::Jump(a),
            // | 2aaa | Pushes pc onto the stack, then jumps to a
            0x2 => Self::Call(a),
            // | 3xbb | Skips next instruction if register X == b
            0x3 => Self::SkipEqualsByte(x, b),
            // | 4xbb | Skips next instruction if register X != b
            0x4 => Self::SkipNotEqualsByte(x, b),
            // # Performs a register-register comparison
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | 9XY0 | Skip next instruction if vX == vY  |
            0x5 => match n {
                0x0 => Self::SkipEquals(x, y),
                _ => Self::Unimplemented(opcode),
            },
            // 6xbb: Loads immediate byte b into register vX
            0x6 => Self::LoadImmediate(x, b),
            // 7xbb: Adds immediate byte b to register vX
            0x7 => Self::AddImmediate(x, b),
            // # Performs ALU operation
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | 8xy0 | X = Y                              |
            // | 8xy1 | X = X | Y                          |
            // | 8xy2 | X = X & Y                          |
            // | 8xy3 | X = X ^ Y                          |
            // | 8xy4 | X = X + Y; Set vF=carry            |
            // | 8xy5 | X = X - Y; Set vF=carry            |
            // | 8xy6 | X = X >> 1                         |
            // | 8xy7 | X = Y - X; Set vF=carry            |
            // | 8xyE | X = X << 1                         |
            0x8 => match n {
                0x0 => Self::Copy(x, y),
                0x1 => Self::Or(x, y),
                0x2 => Self::And(x, y),
                0x3 => Self::Xor(x, y),
                0x4 => Self::Add(x, y),
                0x5 => Self::Sub(x, y),
                0x6 => Self::ShiftRight(x, y),
                0x7 => Self::BackwardsSub(x, y),
                0xE => Self::ShiftLeft(x, y),
                _ => Self::Unimplemented(opcode),
            },
            // # Performs a register-register comparison
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | 9XY0 | Skip next instruction if vX != vY  |
            0x9 => match n {
                0 => Self::SkipNotEquals(x, y),
                _ => Self::Unimplemented(opcode),
            },
            // Aaaa: Load address #a into register I
            0xa => Self::LoadIndirect(a),
            // Baaa: Jump to &adr + v0
            0xb => Self::JumpIndexed(a),
            // Cxbb: Stores a random number & the provided byte into vX
            0xc => Self::Rand(x, b),
            // Dxyn: Draws n-byte sprite to the screen at coordinates (vX, vY)
            0xd => Self::Draw(x, y, n),

            // # Skips instruction on value of keypress
            // |opcode| effect                             |
            // |------|------------------------------------|
            // | eX9e | Skip next instruction if key == #X |
            // | eXa1 | Skip next instruction if key != #X |
            0xe => match b {
                0x9e => Self::SkipEqualsKey(x),
                0xa1 => Self::SkipNotEqualsKey(x),
                _ => Self::Unimplemented(opcode),
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
                0x07 => Self::StoreDelay(x),
                0x0A => Self::WaitForKey(x),
                0x15 => Self::LoadDelay(x),
                0x18 => Self::LoadSound(x),
                0x1E => Self::AddIndirect(x),
                0x29 => Self::LoadSprite(x),
                0x33 => Self::BcdConvert(x),
                0x55 => Self::DmaStore(x),
                0x65 => Self::DmaLoad(x),
                _ => Self::Unimplemented(opcode),
            },
            _ => unreachable!("i somehow mutated from <= 0xf to > 0xf"),
        })
    }
}
