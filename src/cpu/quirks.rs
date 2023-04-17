//! Controls the [Quirks] behavior of the CPU on a granular level.

/// Controls the quirk behavior of the CPU on a granular level.
///
/// `false` is Cosmac-VIP-like behavior
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quirks {
    /// Super Chip: Binary ops in `8xy`(`1`, `2`, `3`) shouldn't set vF to 0
    pub bin_ops: bool,
    /// Super Chip: Shift ops in `8xy`(`6`, `E`) shouldn't source from vY instead of vX
    pub shift: bool,
    /// Super Chip: Draw operations shouldn't pause execution until the next timer tick
    pub draw_wait: bool,
    /// XO-Chip:    Draw operations should wrap from bottom to top and side to side
    pub screen_wrap: bool,
    /// Super Chip: DMA instructions `Fx55`/`Fx65` shouldn't change I to I + x + 1
    pub dma_inc: bool,
    /// Super Chip: Indexed jump instructions should go to `adr` + v`a` where `a` is high nibble of `adr`.
    pub stupid_jumps: bool,
}

impl From<bool> for Quirks {
    fn from(value: bool) -> Self {
        if value {
            Quirks {
                bin_ops: true,
                shift: true,
                draw_wait: true,
                screen_wrap: false,
                dma_inc: true,
                stupid_jumps: true,
            }
        } else {
            Quirks {
                bin_ops: false,
                shift: false,
                draw_wait: false,
                screen_wrap: false,
                dma_inc: false,
                stupid_jumps: false,
            }
        }
    }
}

impl Default for Quirks {
    fn default() -> Self {
        Self::from(false)
    }
}
