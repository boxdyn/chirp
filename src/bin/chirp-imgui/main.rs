#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![allow(dead_code)] // TODO: finish writing the code

mod args;
mod emu;
mod error;
mod gui;

use crate::args::Arguments;
use crate::emu::*;
use crate::gui::*;
use owo_colors::OwoColorize;
use pixels::{Pixels, SurfaceTexture};
use std::result::Result;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

// TODO: Make these configurable in the frontend

const FOREGROUND: &[u8; 4] = &0xFFFF00FF_u32.to_be_bytes();
const BACKGROUND: &[u8; 4] = &0x623701FF_u32.to_be_bytes();
const INIT_SPEED: usize = 10;

struct Application {
    gui: Gui,
    emu: Emulator,
}

fn main() -> Result<(), error::Error> {
    let args = Arguments::parse();
    // Make sure the ROM file exists
    if !args.file.is_file() {
        eprintln!(
            "{} not found. If the file exists, you might not have permission you access it.",
            args.file.display().italic().red()
        );
        return Ok(());
    }

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let size = LogicalSize::new(128 * 8, 64 * 8);
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

    let mut gui = Gui::new(&window, &pixels);
    // set initial parameters
    if let Some(mode) = args.mode {
        gui.menubar.settings.set_mode(mode);
    }

    gui.menubar.settings.set_color(FOREGROUND, BACKGROUND);
    *gui.menubar.settings.target_ipf() = args.speed.unwrap_or(INIT_SPEED);

    let mut app = Application::new(args.into(), gui);

    // Copy quirks from the running Emulator, for consistency
    *app.gui.menubar.settings.quirks() = app.emu.quirks();

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        let toggle_fullscreen = || {
            window.set_fullscreen(if window.fullscreen().is_some() {
                None
            } else {
                Some(winit::window::Fullscreen::Borderless(None))
            })
        };

        let redraw = |gui: &mut Gui, pixels: &mut Pixels| -> Result<(), error::Error> {
            // Prepare gui for redraw
            gui.prepare(&window)?;

            // Render everything together
            pixels.render_with(|encoder, render_target, context| {
                // Render the emulator's screen
                context.scaling_renderer.render(encoder, render_target);
                // Render Dear ImGui
                gui.render(&window, encoder, render_target, context)?;
                Ok(())
            })?;
            Ok(())
        };

        let mut handle_events = |state: &mut Application,
                                 pixels: &mut Pixels,
                                 control_flow: &mut ControlFlow|
         -> Result<(), error::Error> {
            state.gui.handle_event(&window, &event);
            if input.update(&event) {
                use VirtualKeyCode::*;
                match_pressed!( match input {
                    Escape | input.quit() | state.gui.wants(Wants::Quit) => {
                        *control_flow = ControlFlow::Exit;
                        return Ok(());
                    },
                    F1 => state.emu.print_registers(),
                    F2 => state.emu.print_screen()?,
                    F3 => state.emu.dump_screen()?,
                    F4 => state.emu.is_disasm(),
                    F5 => state.emu.pause(),
                    F6 => state.emu.singlestep()?,
                    F7 => state.emu.set_break(),
                    F8 => state.emu.unset_break(),
                    Delete => state.emu.soft_reset(),
                    Insert => state.emu.hard_reset(),
                    F11 => toggle_fullscreen(),
                    LAlt => state.gui.show_menubar(None),
                });
                state.emu.input(&input)?;

                // Apply settings
                if let Some((ipf, quirks, fg, bg)) = state.gui.menubar.settings.applied() {
                    state.emu.ipf = ipf;
                    state.emu.set_quirks(quirks);
                    state.emu.colors = [fg, bg];
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

                state.emu.set_disasm(state.gui.wants(Wants::Disasm));
                if state.gui.wants(Wants::Reset) {
                    state.emu.hard_reset();
                }

                // Run the game loop
                state.emu.update()?;
                state.emu.draw(pixels)?;

                // redraw the window
                window.request_redraw();
            }
            Ok(())
        };

        if let Event::RedrawRequested(_) = event {
            if let Err(e) = redraw(&mut app.gui, &mut pixels) {
                eprintln!("{e}");
                *control_flow = ControlFlow::Exit;
            }
        }

        if let Err(e) = handle_events(&mut app, &mut pixels, control_flow) {
            eprintln!("{e}");
            *control_flow = ControlFlow::Exit;
        }
    });
}

impl Application {
    fn new(emu: Emulator, gui: Gui) -> Self {
        Self { gui, emu }
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
