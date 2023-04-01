// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE.txt for details)

//! Error type for Chirp

use std::ops::Range;

use crate::bus::Region;
use thiserror::Error;

/// Result type, equivalent to [std::result::Result]<T, [enum@Error]>
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for Chirp.
#[derive(Debug, Error)]
pub enum Error {
    /// Represents an unimplemented operation
    #[error("Unrecognized opcode: {word:04x}")]
    UnimplementedInstruction {
        /// The offending word
        word: u16,
    },
    /// The region you asked for was not defined
    #[error("No {region} found on bus")]
    MissingRegion {
        /// The offending [Region]
        region: Region,
    },
    /// Tried to fetch [Range] from bus, received nothing
    #[error("Invalid range {range:04x?} for bus")]
    InvalidBusRange {
        /// The offending [Range]
        range: Range<usize>,
    },
    /// Tried to press a key that doesn't exist
    #[error("Invalid key: {key:X}")]
    InvalidKey {
        /// The offending key
        key: usize,
    },
    /// Tried to get/set an out-of-bounds register
    #[error("Invalid register: v{reg:X}")]
    InvalidRegister {
        /// The offending register
        reg: usize,
    },
    /// Error originated in [std::io]
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    /// Error originated in [std::array::TryFromSliceError]
    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    /// Error originated in [minifb]
    #[error(transparent)]
    MinifbError(#[from] minifb::Error),
}
