//! Error type for chirp-imgui

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// Error originated in [`chirp`]
    #[error(transparent)]
    Chirp(#[from] chirp::error::Error),
    /// Error originated in [`std::io`]
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Error originated in [`pixels`]
    #[error(transparent)]
    Pixels(#[from] pixels::Error),
    /// Error originated in [`pixels`]
    #[error(transparent)]
    PixelsTexture(#[from] pixels::TextureError),
    /// Error originated from [`winit::error::OsError`]
    #[error(transparent)]
    WinitOs(#[from] winit::error::OsError),
    /// Error originated from [`winit::error::ExternalError`]
    #[error(transparent)]
    WinitExternal(#[from] winit::error::ExternalError),
    /// Error originated from [`winit::error::NotSupportedError`]
    #[error(transparent)]
    WinitNotSupported(#[from] winit::error::NotSupportedError),
}
