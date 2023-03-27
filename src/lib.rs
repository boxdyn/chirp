// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE.txt for details)

#![feature(stmt_expr_attributes)]
//! This crate implements a Chip-8 interpreter as if it were a real CPU architecture,
//! to the best of my current knowledge. As it's the first emulator project I've
//! embarked on, and I'm fairly new to Rust, it's going to be rough around the edges.
//!
//! Hopefully, though, you'll find some use in it.

pub mod bus;
pub mod cpu;
pub mod error;
pub mod io;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chip8 {
    pub cpu: cpu::CPU,
    pub bus: bus::Bus,
}

/// Common imports for chumpulator
pub mod prelude {
    pub use super::Chip8;
    use super::*;
    pub use crate::bus;
    pub use bus::{Bus, Read, Region::*, Write};
    pub use cpu::{disassemble::Disassemble, ControlFlags, CPU};
    pub use error::Result;
    pub use io::{UIBuilder, *};
}
