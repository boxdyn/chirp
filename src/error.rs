// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE.txt for details)

//! Error type for Chirp

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unrecognized opcode {word}")]
    UnimplementedInstruction { word: u16 },
    #[error("Math was funky when parsing {word}: {explanation}")]
    FunkyMath { word: u16, explanation: String },
    #[error("No {region} found on bus")]
    MissingRegion { region: String },
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    WindowError(#[from] minifb::Error),
}
