#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![allow(dead_code)] // TODO: finish writing the code
use crate::gui::*;
use chirp::*;
use core::panic;
use pixels::{Pixels, SurfaceTexture};
use std::result::Result;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod error;
mod gui;

const FOREGROUND: &[u8; 4] = &0xFFFF00FF_u32.to_be_bytes();
const BACKGROUND: &[u8; 4] = &0x623701FF_u32.to_be_bytes();

/// The state of the application
#[derive(Debug)]
struct Emulator {
    gui: Gui,
    screen: Bus,
    cpu: CPU,
    ipf: usize,
    colors: [[u8; 4]; 2],
}

fn main() -> Result<(), error::Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let size = LogicalSize::new(128 * 6, 64 * 6);
    let window = WindowBuilder::new()
        .with_title("Chirp")
        .with_inner_size(size)
        .with_always_on_top(true)
        .build(&event_loop)?;
    let mut scale_factor = window.scale_factor();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(128, 64, surface_texture)?
    };

    let mut emu = Emulator::new(
        Gui::new(&window, &pixels),
        bus! {
            Screen  [0x000..0x100],
        },
        CPU::default(),
        10,
    );

    // set initial parameters
    *emu.gui.menubar.settings.target_ipf() = emu.cpu.flags.monotonic.unwrap_or(10);
    *emu.gui.menubar.settings.quirks() = emu.cpu.flags.quirks;
    if let Some(path) = std::env::args().nth(1) {
        emu.cpu.load_program(path)?;
    } else {
        panic!("Supply a rom!");
    }

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        let redraw = |gui: &mut Gui, pixels: &mut Pixels| -> Result<(), error::Error> {
            // Prepare gui for redraw
            gui.prepare(&window)?;

            // Render everything together
            pixels.render_with(|encoder, render_target, context| {
                // Render the world texture
                context.scaling_renderer.render(encoder, render_target);
                // Render Dear ImGui
                gui.render(&window, encoder, render_target, context)?;
                Ok(())
            })?;
            Ok(())
        };

        let mut handle_events = |state: &mut Emulator,
                                 pixels: &mut Pixels,
                                 control_flow: &mut ControlFlow|
         -> Result<(), error::Error> {
            state.gui.handle_event(&window, &event);
            if input.update(&event) {
                use VirtualKeyCode::*;
                match_pressed!( match input {
                    Escape | input.quit() | state.gui.wants_quit() => {
                        *control_flow = ControlFlow::Exit;
                        return Ok(());
                    },
                    F1 => state.cpu.dump(),
                    F2 => state.screen.print_screen()?,
                    F3 => eprintln!("TODO: Dump screen"),
                    F4 => state.cpu.flags.debug(),
                    F5 => state.cpu.flags.pause(),
                    F6 => state.cpu.singlestep(&mut state.screen)?,
                    F7 => state.cpu.set_break(state.cpu.pc()),
                    F8 => state.cpu.unset_break(state.cpu.pc()),
                    Delete => state.cpu.reset(),
                    F11 => window.set_maximized(!window.is_maximized()),
                    LAlt => state.gui.show_menubar(None),
                });
                state.input(&input)?;

                // Apply settings
                if let Some((ipf, quirks)) = state.gui.menubar.settings.applied() {
                    state.ipf = ipf;
                    state.cpu.flags.monotonic = Some(ipf);
                    state.cpu.flags.quirks = quirks;
                }

                // Update the scale factor
                if let Some(factor) = input.scale_factor() {
                    scale_factor = factor;
                }

                // Resize the window
                if let Some(size) = input.window_resized() {
                    if size.width > 0 && size.height > 0 {
                        // Resize the surface texture
                        pixels.resize_surface(size.width, size.height)?;
                        // Resize the world
                        let LogicalSize { width, height } = size.to_logical(scale_factor);
                        pixels.resize_buffer(width, height)?;
                    }
                }

                state.cpu.flags.debug = state.gui.wants_disassembly();
                if state.gui.wants_reset() {
                    state.cpu.reset();
                }

                // Run the game loop
                state.update()?;
                state.draw(pixels)?;

                // redraw the window
                window.request_redraw();
            }
            Ok(())
        };

        if let Event::RedrawRequested(_) = event {
            if let Err(e) = redraw(&mut emu.gui, &mut pixels) {
                eprintln!("{e}");
                *control_flow = ControlFlow::Exit;
            }
        }

        if let Err(e) = handle_events(&mut emu, &mut pixels, control_flow) {
            eprintln!("{e}");
            *control_flow = ControlFlow::Exit;
        }
    });
}

impl Emulator {
    pub fn new(gui: Gui, mem: Bus, cpu: CPU, ipf: usize) -> Self {
        Self {
            gui,
            screen: mem,
            cpu,
            ipf,
            colors: [*FOREGROUND, *BACKGROUND],
        }
    }

    pub fn update(&mut self) -> Result<(), error::Error> {
        self.cpu.multistep(&mut self.screen, self.ipf)?;
        Ok(())
    }

    pub fn draw(&mut self, pixels: &mut Pixels) -> Result<(), error::Error> {
        if let Some(screen) = self.screen.get_region(Screen) {
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
}

#[macro_export]
macro_rules! match_pressed {
    (match $input:ident {$($key:path $( | $cond:expr)? => $action:expr),+ $(,)?}) => {
        $(
            if $input.key_pressed($key) $( || $cond )? {
                $action;
            }
        )+
    };
}

#[macro_export]
macro_rules! match_released {
    (match $input:ident {$($key:path $( | $cond:expr)? => $action:expr),+ $(,)?}) => {
        $(
            if $input.key_released($key) $( || $cond )? {
                $action;
            }
        )+
    };
}
