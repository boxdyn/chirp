// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! Error type for Chirp

pub mod any_range;
use any_range::AnyRange;

use crate::cpu::mem::Region;
use thiserror::Error;

/// Result type, equivalent to [std::result::Result]<T, [enum@Error]>
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for Chirp.
#[derive(Debug, Error)]
pub enum Error {
    /// Represents a breakpoint being hit
    #[error("breakpoint hit: {addr:03x} ({next:04x})")]
    BreakpointHit {
        /// The address of the breakpoint
        addr: u16,
        /// The instruction after the breakpoint
        next: u16,
    },
    /// Represents an unimplemented operation
    #[error("opcode {word:04x} not recognized")]
    UnimplementedInstruction {
        /// The offending word
        word: u16,
    },
    /// The region you asked for was not defined
    #[error("region {region} is not present on bus")]
    MissingRegion {
        /// The offending [Region]
        region: Region,
    },
    /// Tried to fetch data at [AnyRange] from bus, received nothing
    #[error("range {range:04x?} is not present on bus")]
    InvalidAddressRange {
        /// The offending [AnyRange]
        range: AnyRange<usize>,
    },
    /// Tried to press a key that doesn't exist
    #[error("tried to press key {key:X} which does not exist")]
    InvalidKey {
        /// The offending key
        key: usize,
    },
    /// Tried to get/set an out-of-bounds register
    #[error("tried to access register v{reg:X} which does not exist")]
    InvalidRegister {
        /// The offending register
        reg: usize,
    },
    /// Tried to convert string into mode, but it did not match.
    #[error("no suitable conversion of \"{mode}\" into Mode")]
    InvalidMode {
        /// The string which failed to become a mode
        mode: String,
    },
    /// Error originated in [std::io]
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    /// Error originated in [std::array::TryFromSliceError]
    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[cfg(feature = "minifb")]
    /// Error originated in [minifb]
    #[error(transparent)]
    MinifbError(#[from] minifb::Error),
}
