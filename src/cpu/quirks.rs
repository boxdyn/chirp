//! Controls the authenticity behavior of the CPU on a granular level.
use super::Mode;
/// Controls the authenticity behavior of the CPU on a granular level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quirks {
    /// Binary ops in `8xy`(`1`, `2`, `3`) shouldn't set vF to 0
    pub bin_ops: bool,
    /// Shift ops in `8xy`(`6`, `E`) shouldn't source from vY instead of vX
    pub shift: bool,
    /// Draw operations shouldn't pause execution until the next timer tick
    pub draw_wait: bool,
    /// DMA instructions `Fx55`/`Fx65` shouldn't change I to I + x + 1
    pub dma_inc: bool,
    /// Indexed jump instructions should go to `adr` + v`a` where `a` is high nibble of `adr`.
    pub stupid_jumps: bool,
}

impl From<bool> for Quirks {
    fn from(value: bool) -> Self {
        if value {
            Quirks {
                bin_ops: true,
                shift: true,
                draw_wait: true,
                dma_inc: true,
                stupid_jumps: true,
            }
        } else {
            Quirks {
                bin_ops: false,
                shift: false,
                draw_wait: false,
                dma_inc: false,
                stupid_jumps: false,
            }
        }
    }
}

impl From<Mode> for Quirks {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Chip8 => false.into(),
            Mode::SChip => true.into(),
            Mode::XOChip => Self {
                bin_ops: true,
                shift: false,
                draw_wait: true,
                dma_inc: false,
                stupid_jumps: false,
            },
        }
    }
}

impl Default for Quirks {
    fn default() -> Self {
        Self::from(false)
    }
}
