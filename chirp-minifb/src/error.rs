//! Error type for chirp-minifb

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    /// Error originated in [`chirp`]
    #[error(transparent)]
    Chirp(#[from] chirp::error::Error),
    /// Error originated in [`std::io`]
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Error originated in [`minifb`]
    #[error(transparent)]
    Minifb(#[from] minifb::Error),
}
