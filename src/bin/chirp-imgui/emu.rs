//! The emulator's state, including the screen data

use super::{error, BACKGROUND, FOREGROUND};
use chirp::{Grab, Screen, CPU};
use pixels::Pixels;
use std::path::{Path, PathBuf};
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

/// The state of the application
#[derive(Debug)]
pub struct Emulator {
    screen: Screen,
    cpu: CPU,
    pub ipf: usize,
    pub rom: PathBuf,
    pub colors: [[u8; 4]; 2],
}

impl Emulator {
    /// Constructs a new CPU, with the provided ROM loaded at 0x0200
    ///
    /// # Panics
    /// Panics if the provided ROM does not exist
    pub fn new(ipf: usize, rom: impl AsRef<Path>) -> Self {
        let screen = Screen::default();
        let mut cpu = CPU::default();
        cpu.load_program(&rom).expect("Loaded file MUST exist.");
        Self {
            cpu,
            ipf,
            rom: rom.as_ref().into(),
            screen,
            colors: [*FOREGROUND, *BACKGROUND],
        }
    }
    /// Runs a single epoch
    pub fn update(&mut self) -> Result<(), error::Error> {
        self.cpu.multistep(&mut self.screen, self.ipf)?;
        Ok(())
    }
    /// Rasterizes the screen into a [Pixels] buffer
    pub fn draw(&mut self, pixels: &mut Pixels) -> Result<(), error::Error> {
        if let Some(screen) = self.screen.grab(..) {
            let len_log2 = screen.len().ilog2() / 2;
            #[allow(unused_variables)]
            let (width, height) = (2u32.pow(len_log2 + 2), 2u32.pow(len_log2 + 1));
            pixels.resize_buffer(width, height)?;
            for (idx, pixel) in pixels.frame_mut().iter_mut().enumerate() {
                let (byte, bit, component) = (idx >> 5, (idx >> 2) % 8, idx & 0b11);
                *pixel = if screen[byte] & (0x80 >> bit) > 0 {
                    self.colors[0][component]
                } else {
                    self.colors[1][component]
                }
            }
        }
        Ok(())
    }
    /// Processes keyboard input for the Emulator
    pub fn input(&mut self, input: &WinitInputHelper) -> Result<(), error::Error> {
        const KEYMAP: [VirtualKeyCode; 16] = [
            VirtualKeyCode::X,
            VirtualKeyCode::Key1,
            VirtualKeyCode::Key2,
            VirtualKeyCode::Key3,
            VirtualKeyCode::Q,
            VirtualKeyCode::W,
            VirtualKeyCode::E,
            VirtualKeyCode::A,
            VirtualKeyCode::S,
            VirtualKeyCode::D,
            VirtualKeyCode::Z,
            VirtualKeyCode::C,
            VirtualKeyCode::Key4,
            VirtualKeyCode::R,
            VirtualKeyCode::F,
            VirtualKeyCode::V,
        ];
        for (id, &key) in KEYMAP.iter().enumerate() {
            if input.key_released(key) {
                self.cpu.release(id)?;
            }
            if input.key_pressed(key) {
                self.cpu.press(id)?;
            }
        }
        Ok(())
    }

    pub fn quirks(&mut self) -> chirp::Quirks {
        self.cpu.flags.quirks
    }
    pub fn set_quirks(&mut self, quirks: chirp::Quirks) {
        self.cpu.flags.quirks = quirks
    }
    /// Prints the CPU registers and cycle count to stderr
    pub fn print_registers(&self) {
        self.cpu.dump()
    }
    /// Prints the screen (using the highest resolution available printer) to stdout
    pub fn print_screen(&self) -> Result<(), error::Error> {
        self.screen.print_screen();
        Ok(())
    }
    /// Dumps the raw screen bytes to a file named `{rom}_{cycle}.bin`,
    /// or, failing that, `screen_dump.bin`
    pub fn dump_screen(&self) -> Result<(), error::Error> {
        let mut path = PathBuf::new().join(format!(
            "{}_{}.bin",
            self.rom.file_stem().unwrap_or_default().to_string_lossy(),
            self.cpu.cycle()
        ));
        path.set_extension("bin");
        if std::fs::write(&path, self.screen.as_slice()).is_ok() {
            eprintln!("Saved to {}", &path.display());
        } else if std::fs::write("screen_dump.bin", self.screen.as_slice()).is_ok() {
            eprintln!("Saved to screen_dump.bin");
        } else {
            eprintln!("Failed to dump screen to file.")
        }
        Ok(())
    }
    /// Sets live disassembly
    pub fn set_disasm(&mut self, enabled: bool) {
        self.cpu.flags.debug = enabled;
    }
    /// Checks live disassembly
    pub fn is_disasm(&mut self) {
        eprintln!(
            "Live Disassembly {}abled",
            if self.cpu.flags.debug { "En" } else { "Dis" }
        );
    }
    /// Toggles emulator pause
    pub fn pause(&mut self) {
        self.cpu.flags.pause();
        eprintln!("{}aused", if self.cpu.flags.pause { "P" } else { "Unp" });
    }
    /// Single-steps the emulator, pausing afterward
    pub fn singlestep(&mut self) -> Result<(), error::Error> {
        self.cpu.singlestep(&mut self.screen)?;
        Ok(())
    }
    /// Sets a breakpoint at the current address
    pub fn set_break(&mut self) {
        self.cpu.set_break(self.cpu.pc());
        eprintln!("Set breakpoint at {}", self.cpu.pc());
    }
    /// Unsets a breakpoint at the current address
    pub fn unset_break(&mut self) {
        self.cpu.unset_break(self.cpu.pc());
        eprintln!("Unset breakpoint at {}", self.cpu.pc());
    }
    /// Soft-resets the CPU, keeping the program in memory
    pub fn soft_reset(&mut self) {
        self.cpu.reset();
        self.screen.clear();
        eprintln!("Soft Reset");
    }

    /// Creates a new CPU with the current CPU's flags
    pub fn hard_reset(&mut self) {
        self.cpu.reset();
        self.screen.clear();
        // keep the flags
        let flags = self.cpu.flags.clone();
        // instantiate a completely new CPU, and reload the ROM from disk
        self.cpu = CPU::default();
        self.cpu.flags = flags;
        self.cpu
            .load_program(&self.rom)
            .expect("Previously loaded ROM no longer exists (was it moved?)");
        eprintln!("Hard Reset");
    }
}
