// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! A disassembler for Chip-8 opcodes
use super::Insn;
use imperative_rs::InstructionSet;
use owo_colors::{OwoColorize, Style};

/// Disassembles Chip-8 instructions
pub trait Disassembler {
    /// Disassemble a single instruction
    fn once(&self, insn: u16) -> String;
}

/// Disassembles Chip-8 instructions, printing them in the provided [owo_colors::Style]s
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dis {
    /// Styles invalid instructions
    pub invalid: Style,
    /// Styles valid instruction
    pub normal: Style,
}

impl Default for Dis {
    fn default() -> Self {
        Self {
            invalid: Style::new().bold().red(),
            normal: Style::new().green(),
        }
    }
}

impl Disassembler for Dis {
    fn once(&self, insn: u16) -> String {
        if let Ok((_, insn)) = Insn::decode(&insn.to_be_bytes()) {
            format!("{}", insn.style(self.normal))
        } else {
            format!("{}", format_args!("inval  {insn:04x}").style(self.invalid))
        }
    }
}
