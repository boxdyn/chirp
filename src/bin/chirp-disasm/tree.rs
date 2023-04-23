#![allow(dead_code)]
#![allow(unused_variables)]
use std::collections::HashSet;

use chirp::cpu::instruction::Insn;

type Adr = usize;

/// Represents the kinds of control flow an instruction can take
pub enum DisNode {
    Branch {
        addr: Adr,
        insn: Insn,
        a: Box<DisNode>,
        b: Box<DisNode>,
    },
    Continue {
        addr: Adr,
        insn: Insn,
        next: Box<DisNode>,
    },
    Merge {
        addr: Adr,
        insn: Insn,
    },
    End(Insn),
    Invalid,
}

impl DisNode {
    pub fn travel(
        mem: &[u8],
        visited: &mut HashSet<Adr>,
        current: Adr,
    ) -> Result<DisNode, chirp::Error> {
        use DisNode::*;

        // decode an insn at the current Adr
        // classify the insn
        // If the instruction is invalid, emit an Invalid token
        // If the instruction is already visited, emit a Merge token
        // If the instruction is a ret instruction, emit a Merge token
        // If the instruction is any other instruction, emit a Continue token
        // If the instruction is a branch to current, emit an End token
        // If the instruction is a branch instruction, recursively follow each branch

        Ok(End(Insn::cls))
    }
}
