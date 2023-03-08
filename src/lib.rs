#![feature(let_chains, stmt_expr_attributes)]
/*!
This crate implements a Chip-8 interpreter as if it were a real CPU architecture,
to the best of my current knowledge. As it's the first emulator project I've
embarked on, and I'm fairly new to Rust, it's going to be rough around the edges.

Hopefully, though, you'll find some use in it.
 */

pub mod bus;
pub mod cpu;
pub mod mem;

pub mod dump;
pub mod error;
pub mod screen;

/// Common imports for rumpulator
pub mod prelude {
    use super::*;
    pub use crate::bus;
    pub use bus::{Bus, BusConnectible};
    pub use cpu::{disassemble::Disassemble, CPU};
    pub use dump::{BinDumpable, Dumpable};
    pub use mem::Mem;
    pub use screen::Screen;
}
