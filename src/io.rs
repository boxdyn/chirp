//! Platform-specific IO/UI code, and some debug functionality.
//! TODO: Break this into its own crate.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::{
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
    pub fn new(height: usize, width: usize) -> Self {
        UIBuilder {
            width,
            height,
            ..Default::default()
        }
    }
    pub fn rom(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.rom = Some(path.as_ref().into());
        self
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
    pub fn render(&mut self, window: &mut Window, bus: &Bus) {
        if let Some(screen) = bus.get_region(Region::Screen) {
            for (idx, byte) in screen.iter().enumerate() {
                for bit in 0..8 {
                    self.buffer[8 * idx + bit] = if byte & (1 << 7 - bit) as u8 != 0 {
                        self.format.fg
                    } else {
                        self.format.bg
                    }
                }
            }
        }
        //TODO: NOT THIS
        window
            .update_with_buffer(&self.buffer, self.width, self.height)
            .expect("The window manager should update the buffer.");
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
}

impl UI {
    pub fn frame(&mut self, ch8: &mut Chip8) -> Option<()> {
        {
            if ch8.cpu.flags.pause {
                self.window.set_title("Chirp  ⏸")
            } else {
                self.window.set_title("Chirp  ▶");
            }
            // update framebuffer
            self.fb.render(&mut self.window, &mut ch8.bus);
        }
        Some(())
    }

    pub fn keys(&mut self, ch8: &mut Chip8) -> Option<()> {
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
        use crate::io::Region::*;
        for key in get_keys_released() {
            ch8.cpu.release(identify_key(key));
        }
        // handle keybinds for the UI
        for key in get_keys_pressed() {
            use Key::*;
            match key {
                F1 | Comma => ch8.cpu.dump(),
                F2 | Period => ch8
                    .bus
                    .print_screen()
                    .expect("The 'screen' memory region should exist"),
                F3 => {
                    debug_dump_screen(&ch8, &self.rom).expect("Unable to write debug screen dump");
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
                    ch8.cpu.singlestep(&mut ch8.bus);
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
                Escape => return None,
                key => ch8.cpu.press(identify_key(key)),
            }
        }
        self.keyboard = self.window.get_keys();
        Some(())
    }
}

pub const KEYMAP: [Key; 16] = [
    Key::X,
    Key::Key1,
    Key::Key2,
    Key::Key3,
    Key::Q,
    Key::W,
    Key::E,
    Key::A,
    Key::S,
    Key::D,
    Key::Z,
    Key::C,
    Key::Key4,
    Key::R,
    Key::F,
    Key::V,
];

pub fn identify_key(key: Key) -> usize {
    match key {
        Key::Key1 => 0x1,
        Key::Key2 => 0x2,
        Key::Key3 => 0x3,
        Key::Key4 => 0xc,
        Key::Q => 0x4,
        Key::W => 0x5,
        Key::E => 0x6,
        Key::R => 0xD,
        Key::A => 0x7,
        Key::S => 0x8,
        Key::D => 0x9,
        Key::F => 0xE,
        Key::Z => 0xA,
        Key::X => 0x0,
        Key::C => 0xB,
        Key::V => 0xF,
        _ => 0x10,
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
