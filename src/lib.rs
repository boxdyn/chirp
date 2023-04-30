// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)
#![cfg_attr(feature = "nightly", feature(no_coverage))]
#![deny(missing_docs, clippy::all)]
//! This crate implements a Chip-8 interpreter as if it were a real CPU architecture,
//! to the best of my current knowledge. As it's the first emulator project I've
//! embarked on, and I'm fairly new to Rust, it's going to be rough around the edges.
//!
//! Hopefully, though, you'll find some use in it.

pub mod cpu;
pub mod error;
pub mod traits;

// Common imports for Chirp
pub use cpu::{
    bus::{Bus, Region::*},
    flags::Flags,
    instruction::disassembler::{Dis, Disassembler},
    mode::Mode,
    quirks::Quirks,
    CPU,
};
pub use error::{Error, Result};
pub use traits::auto_cast::{AutoCast, Grab};

/// Holds the state of a Chip-8
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chip8 {
    /// Contains the registers, flags, and operating state for a single Chip-8
    pub cpu: cpu::CPU,
    /// Contains the memory of a chip-8
    pub bus: cpu::bus::Bus,
}
