#![allow(unused_imports)]
use chirp::*;
#[cfg(features = "iced")]
use iced::{
    executor, time, window, Alignment, Application, Command, Element, Length, Settings,
    Subscription,
};

#[cfg(features = "iced")]
fn main() -> iced::Result {
    Ok(())
}

#[cfg(not(features = "iced"))]
fn main() -> Result<()> {
    Ok(())
}

/// TODO: `impl Application for Emulator {}`
#[derive(Clone, Debug, Default, PartialEq)]
struct Emulator {
    mem: Bus,
    cpu: CPU,
    fps: f64,
    ipf: usize,
}
