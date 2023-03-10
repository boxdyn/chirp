//! Error type for chumpulator

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    #[error("Unrecognized opcode {word}")]
    UnimplementedInstruction { word: u16 },
    #[error("Math was funky when parsing {word}: {explanation}")]
    FunkyMath { word: u16, explanation: String },
}
