//!

use crate::{bus::{Bus, Region}, cpu::CPU, error::Result};
use minifb::*;

#[derive(Clone, Copy, Debug)]
pub struct WindowBuilder {
    pub width: usize,
    pub height: usize,
    pub name: Option<&'static str>,
    pub window_options: WindowOptions,
}

impl WindowBuilder {
    pub fn new(height: usize, width: usize) -> Self {
        WindowBuilder {
            width,
            height,
            ..Default::default()
        }
    }
    pub fn build(self) -> Result<Window> {
        Ok(Window::new(
            self.name.unwrap_or_default(),
            self.width,
            self.height,
            self.window_options,
        )?)
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        WindowBuilder {
            width: 64,
            height: 32,
            name: Some("Chip-8 Interpreter"),
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

pub struct FrameBufferFormat {
    pub fg: u32,
    pub bg: u32
}

impl Default for FrameBufferFormat {
    fn default() -> Self {
        FrameBufferFormat { fg: 0x0011a434, bg: 0x001E2431 }
    }
}

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
            .expect("The window manager has encountered an issue I don't want to deal with");
    }
}

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

/// Gets keys from the Window, and feeds them directly to the CPU
pub fn get_keys(window: &mut Window, cpu: &mut CPU) {
    cpu.release();
    window
        .get_keys()
        .iter()
        .for_each(|key| cpu.press(identify_key(*key)));
}
