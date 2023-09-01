#![allow(dead_code)]
#![allow(unused_variables)]
use std::{
    collections::{BTreeMap, HashSet},
    fmt::{Display, LowerHex},
    rc::{Rc, Weak},
};

use chirp::cpu::instruction::Insn;
use imperative_rs::InstructionSet;
use owo_colors::OwoColorize;

type Adr = usize;

/// Represents the kinds of control flow an instruction can take
#[derive(Clone, Debug, Default)]
pub enum DisNodeContents {
    Subroutine {
        insn: Insn,
        jump: Rc<DisNode>,
        ret: Rc<DisNode>,
    },
    Branch {
        insn: Insn,
        next: Rc<DisNode>,
        jump: Rc<DisNode>,
    },
    Continue {
        insn: Insn,
        next: Rc<DisNode>,
    },
    End {
        insn: Insn,
    },
    RelBranch {
        insn: Insn,
    },
    Merge {
        insn: Insn,
        back: Option<Weak<DisNode>>,
    },
    PendingMerge {
        insn: Insn,
    },
    #[default]
    Invalid,
}

/// Represents the kinds of control flow an instruction can take
#[derive(Clone, Debug)]
pub struct DisNode {
    pub contents: DisNodeContents,
    pub addr: Adr,
    pub depth: usize,
}

impl DisNode {
    pub fn traverse(
        mem: &[u8],
        nodes: &mut BTreeMap<Adr, Weak<DisNode>>,
        addr: Adr,
    ) -> Rc<DisNode> {
        Self::tree_recurse(mem, &mut Default::default(), nodes, addr, 0)
    }
    pub fn tree_recurse(
        mem: &[u8],
        visited: &mut HashSet<Adr>,
        nodes: &mut BTreeMap<Adr, Weak<DisNode>>,
        addr: Adr,
        depth: usize,
    ) -> Rc<DisNode> {
        use DisNodeContents::*;
        // Try to decode an instruction. If the instruction is invalid, fail early.
        let Ok((len, insn)) = Insn::decode(&mem[addr..]) else {
            return Rc::new(DisNode {
                contents: Invalid,
                addr,
                depth,
            });
        };
        let mut next = DisNode {
            contents: {
                match insn {
                    // instruction is already visited, but the branch isn't guaranteed to be in the tree yet
                    _ if !visited.insert(addr) => PendingMerge { insn },

                    Insn::ret | Insn::halt => End { insn },

                    // A branch to the current address will halt the machine
                    Insn::jmp { A } | Insn::call { A } if A as usize == addr => End { insn },

                    Insn::jmp { A } => Continue {
                        insn,
                        next: DisNode::tree_recurse(mem, visited, nodes, A as usize, depth),
                    },

                    Insn::call { A } => Branch {
                        insn,
                        jump: DisNode::tree_recurse(mem, visited, nodes, A as usize, depth + 1),
                        next: DisNode::tree_recurse(mem, visited, nodes, addr + len, depth),
                    },

                    // If the instruction is a skip instruction, first visit the next instruction,
                    // then visit the skip instruction. This preserves visitor order.
                    Insn::seb { .. }
                    | Insn::sneb { .. }
                    | Insn::se { .. }
                    | Insn::sne { .. }
                    | Insn::sek { .. } => Branch {
                        insn,
                        // FIXME: If the next instruction is Long I, this will just break
                        next: DisNode::tree_recurse(mem, visited, nodes, addr + len, depth),
                        jump: DisNode::tree_recurse(mem, visited, nodes, addr + len + 2, depth + 1),
                    },

                    // Relative branch prediction is out of scope right now
                    Insn::jmpr { .. } => RelBranch { insn },

                    // If the instruction is any other instruction, emit a Continue token
                    _ => Continue {
                        insn,
                        next: DisNode::tree_recurse(mem, visited, nodes, addr + len, depth),
                    },
                }
            },
            addr,
            depth,
        };
        // Resolve pending merges
        if let PendingMerge { insn } = next.contents {
            next.contents = Merge {
                insn,
                back: nodes.get(&addr).cloned(),
            }
        }
        let next = Rc::new(next);
        nodes.insert(addr, Rc::downgrade(&next));
        next
    }
}

impl Display for DisNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DisNodeContents::*;
        write!(f, "\n{:04x}: ", self.addr,)?;
        for indent in 0..self.depth {
            Display::fmt(&"│  ".bright_magenta(), f)?;
        }
        match &self.contents {
            Subroutine { insn, ret, jump } => write!(f, "{insn}{jump}{ret}"),
            Branch { insn, next, jump } => write!(f, "{insn}{jump}{next}"),
            Continue { insn, next } => write!(f, "{insn}{next}"),
            RelBranch { insn } => Display::fmt(&insn.underline(), f),
            PendingMerge { insn } | Merge { insn, .. } => write!(
                f,
                "{}",
                format_args!("{}; ...", insn).italic().bright_black()
            ),
            End { insn } => Display::fmt(insn, f),
            Invalid => Display::fmt(&"Invalid".bold().red(), f),
        }
    }
}

impl LowerHex for DisNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DisNodeContents::*;
        match self.contents {
            Subroutine { insn, .. }
            | Branch { insn, .. }
            | Continue { insn, .. }
            | Merge { insn, .. }
            | End { insn }
            | RelBranch { insn }
            | PendingMerge { insn } => {
                LowerHex::fmt(&u32::from(&insn).bright_black(), f)?;
                f.write_str(" ")?;
                Display::fmt(&insn.cyan(), f)
            }
            Invalid => Display::fmt("Invalid", f),
        }
    }
}
