// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE.txt for details)
#![allow(missing_docs)]
//! Platform-specific IO/UI code, and some debug functionality.
//! TODO: Destroy this all.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    time::Instant,
};

use chirp::{
    bus::{Bus, Region},
    error::Result,
    Chip8,
};
use minifb::*;

#[derive(Clone, Debug)]
pub struct UIBuilder {
    pub width: usize,
    pub height: usize,
    pub name: Option<&'static str>,
    pub rom: Option<PathBuf>,
    pub window_options: WindowOptions,
}

impl UIBuilder {
    #[allow(dead_code)] // this code is used in tests thank you
    pub fn new(width: usize, height: usize, rom: impl AsRef<Path>) -> Self {
        UIBuilder {
            width,
            height,
            rom: Some(rom.as_ref().to_owned()),
            ..Default::default()
        }
    }
    pub fn build(&self) -> Result<UI> {
        let ui = UI {
            window: Window::new(
                self.name.unwrap_or_default(),
                self.width,
                self.height,
                self.window_options,
            )?,
            keyboard: Default::default(),
            fb: Default::default(),
            rom: self.rom.to_owned().unwrap_or_default(),
            time: Instant::now(),
        };
        Ok(ui)
    }
}

impl Default for UIBuilder {
    fn default() -> Self {
        UIBuilder {
            width: 64,
            height: 32,
            name: Some("Chip-8 Interpreter"),
            rom: None,
            window_options: WindowOptions {
                title: true,
                resize: false,
                scale: Scale::X16,
                scale_mode: ScaleMode::AspectRatioStretch,
                none: true,
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameBufferFormat {
    pub fg: u32,
    pub bg: u32,
}

impl Default for FrameBufferFormat {
    fn default() -> Self {
        FrameBufferFormat {
            fg: 0x0011a434,
            bg: 0x001E2431,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameBuffer {
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    format: FrameBufferFormat,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        FrameBuffer {
            buffer: vec![0x00be4d; width * height],
            width,
            height,
            format: Default::default(),
        }
    }
    pub fn render(&mut self, window: &mut Window, bus: &Bus) -> Result<()> {
        if let Some(screen) = bus.get_region(Region::Screen) {
            for (idx, byte) in screen.iter().enumerate() {
                for bit in 0..8 {
                    self.buffer[8 * idx + bit] = if byte & (1 << (7 - bit)) as u8 != 0 {
                        self.format.fg
                    } else {
                        self.format.bg
                    }
                }
            }
        }
        window.update_with_buffer(&self.buffer, self.width, self.height)?;
        Ok(())
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new(64, 32)
    }
}

#[derive(Debug)]
pub struct UI {
    window: Window,
    keyboard: Vec<Key>,
    fb: FrameBuffer,
    rom: PathBuf,
    time: Instant,
}

impl UI {
    pub fn frame(&mut self, ch8: &mut Chip8) -> Result<bool> {
        if ch8.cpu.flags.pause {
            self.window.set_title("Chirp ⏸")
        } else {
            self.window.set_title(&format!(
                "Chirp  ▶ {:02.02}",
                (1.0 / self.time.elapsed().as_secs_f64())
            ));
        }
        if !self.window.is_open() {
            return Ok(false);
        }
        self.time = Instant::now();
        // update framebuffer
        self.fb.render(&mut self.window, &ch8.bus)?;
        Ok(true)
    }

    pub fn keys(&mut self, ch8: &mut Chip8) -> Result<bool> {
        // TODO: Remove this hacky workaround for minifb's broken get_keys_* functions.
        let get_keys_pressed = || {
            self.window
                .get_keys()
                .into_iter()
                .filter(|key| !self.keyboard.contains(key))
        };
        let get_keys_released = || {
            self.keyboard
                .clone()
                .into_iter()
                .filter(|key| !self.window.get_keys().contains(key))
        };
        use crate::ui::Region::*;
        for key in get_keys_released() {
            if let Some(key) = identify_key(key) {
                ch8.cpu.release(key)?;
            }
        }
        // handle keybinds for the UI
        for key in get_keys_pressed() {
            use Key::*;
            match key {
                F1 | Comma => ch8.cpu.dump(),
                F2 | Period => ch8.bus.print_screen()?,
                F3 => {
                    debug_dump_screen(ch8, &self.rom).expect("Unable to write debug screen dump");
                }
                F4 | Slash => {
                    eprintln!("Debug {}.", {
                        ch8.cpu.flags.debug();
                        if ch8.cpu.flags.debug {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    })
                }
                F5 | Backslash => eprintln!("{}.", {
                    ch8.cpu.flags.pause();
                    if ch8.cpu.flags.pause {
                        "Paused"
                    } else {
                        "Unpaused"
                    }
                }),
                F6 | Enter => {
                    eprintln!("Step");
                    ch8.cpu.singlestep(&mut ch8.bus)?;
                }
                F7 => {
                    eprintln!("Set breakpoint {:03x}.", ch8.cpu.pc());
                    ch8.cpu.set_break(ch8.cpu.pc());
                }
                F8 => {
                    eprintln!("Unset breakpoint {:03x}.", ch8.cpu.pc());
                    ch8.cpu.unset_break(ch8.cpu.pc());
                }
                F9 | Delete => {
                    eprintln!("Soft reset state.cpu {:03x}", ch8.cpu.pc());
                    ch8.cpu.soft_reset();
                    ch8.bus.clear_region(Screen);
                }
                Escape => return Ok(false),
                key => {
                    if let Some(key) = identify_key(key) {
                        ch8.cpu.press(key)?;
                    }
                }
            }
        }
        self.keyboard = self.window.get_keys();
        Ok(true)
    }
}

pub fn identify_key(key: Key) -> Option<usize> {
    match key {
        Key::Key1 => Some(0x1),
        Key::Key2 => Some(0x2),
        Key::Key3 => Some(0x3),
        Key::Key4 => Some(0xc),
        Key::Q => Some(0x4),
        Key::W => Some(0x5),
        Key::E => Some(0x6),
        Key::R => Some(0xD),
        Key::A => Some(0x7),
        Key::S => Some(0x8),
        Key::D => Some(0x9),
        Key::F => Some(0xE),
        Key::Z => Some(0xA),
        Key::X => Some(0x0),
        Key::C => Some(0xB),
        Key::V => Some(0xF),
        _ => None,
    }
}

pub fn debug_dump_screen(ch8: &Chip8, rom: &Path) -> Result<()> {
    let path = PathBuf::new()
        .join("src/cpu/tests/screens/")
        .join(if rom.is_absolute() {
            Path::new("unknown/")
        } else {
            rom.file_name().unwrap_or(OsStr::new("unknown")).as_ref()
        })
        .join(format!("{}.bin", ch8.cpu.cycle()));
    std::fs::write(
        &path,
        ch8.bus
            .get_region(Region::Screen)
            .expect("Region::Screen should exist"),
    )
    .unwrap_or_else(|_| {
        std::fs::write(
            "screendump.bin",
            ch8.bus
                .get_region(Region::Screen)
                .expect("Region::Screen should exist"),
        )
        .ok(); // lmao
    });
    eprintln!("Saved to {}", &path.display());
    Ok(())
}
