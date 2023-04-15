//! Represents flags that aid in implementation but aren't a part of the Chip-8 spec

use super::{Mode, Quirks};

/// Represents flags that aid in operation, but aren't inherent to the CPU
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flags {
    /// Set when debug (live disassembly) mode enabled
    pub debug: bool,
    /// Set when the emulator is paused by the user and should not update
    pub pause: bool,
    /// Set when the emulator is waiting for a keypress
    pub keypause: bool,
    /// Set when the emulator is waiting for a frame to be drawn
    pub draw_wait: bool,
    /// Set when the emulator is in high-res mode
    pub draw_mode: bool,
    /// Set to the last key that's been *released* after a keypause
    pub lastkey: Option<usize>,
    /// Represents the current emulator [Mode]
    pub mode: Mode,
    /// Represents the set of emulator [Quirks] to enable, independent of the [Mode]
    pub quirks: Quirks,
    /// Represents the number of instructions to run per tick of the internal timer
    pub monotonic: Option<usize>,
}

impl Flags {
    /// Toggles debug mode
    ///
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// assert_eq!(true, cpu.flags.debug);
    /// // Toggle debug mode
    /// cpu.flags.debug();
    /// assert_eq!(false, cpu.flags.debug);
    /// ```
    pub fn debug(&mut self) {
        self.debug = !self.debug
    }

    /// Toggles pause
    ///
    /// # Examples
    /// ```rust
    /// # use chirp::*;
    /// let mut cpu = CPU::default();
    /// assert_eq!(false, cpu.flags.pause);
    /// // Pause the cpu
    /// cpu.flags.pause();
    /// assert_eq!(true, cpu.flags.pause);
    /// ```
    pub fn pause(&mut self) {
        self.pause = !self.pause
    }
}
