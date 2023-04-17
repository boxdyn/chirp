// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE.txt for details)
#![cfg_attr(feature = "unstable", feature(no_coverage))]
#![deny(missing_docs, clippy::all)]
//! This crate implements a Chip-8 interpreter as if it were a real CPU architecture,
//! to the best of my current knowledge. As it's the first emulator project I've
//! embarked on, and I'm fairly new to Rust, it's going to be rough around the edges.
//!
//! Hopefully, though, you'll find some use in it.

pub mod cpu;
pub mod error;

// Common imports for Chirp
pub use cpu::{
    bus::{Bus, Get, ReadWrite, Region::*},
    disassembler::{Dis, Disassembler},
    flags::Flags,
    mode::Mode,
    quirks::Quirks,
    CPU,
};
pub use error::{Error, Result};

/// Holds the state of a Chip-8
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chip8 {
    /// Contains the registers, flags, and operating state for a single Chip-8
    pub cpu: cpu::CPU,
    /// Contains the memory of a chip-8
    pub bus: cpu::bus::Bus,
}
