//! Represents the Dear Imgui
//!
//! Adapted from the [ImGui-winit Example]
//!
//! [ImGui-winit Example]: https://github.com/parasyte/pixels/blob/main/examples/imgui-winit/src/gui.rs

use pixels::{wgpu, PixelsContext};
use std::time::Instant;

mod about;
mod menubar;
use menubar::Menubar;

/// Lays out the imgui widgets for a thing
pub trait Drawable {
    // Lay out the ImGui widgets for this thing
    fn draw(&mut self, ui: &imgui::Ui);
}

/// Holds state of GUI
pub(crate) struct Gui {
    imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    last_frame: Instant,
    last_cursor: Option<imgui::MouseCursor>,
    pub menubar: Menubar,
}

/// Queries the state of the [Gui]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Wants {
    Quit,
    Disasm,
    SoftReset,
    HardReset,
    Reset,
}

impl std::fmt::Debug for Gui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gui")
            .field("imgui", &self.imgui)
            .field("platform", &self.platform)
            .field("last_frame", &self.last_frame)
            .field("last_cursor", &self.last_cursor)
            .field("menubar", &self.menubar)
            .finish_non_exhaustive()
    }
}

impl Gui {
    pub fn new(window: &winit::window::Window, pixels: &pixels::Pixels) -> Self {
        // Create Dear Imgui context
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        // winit init
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Locked(2.2),
        );

        // Configure fonts
        let dpi_scale = window.scale_factor();
        let font_size = (13.0 * dpi_scale) as f32;
        imgui.io_mut().font_global_scale = (1.0 / dpi_scale) as f32;
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    size_pixels: font_size,
                    oversample_h: 2,
                    oversample_v: 2,
                    pixel_snap_h: true,
                    ..Default::default()
                }),
            }]);

        // Create WGPU renderer
        let renderer = imgui_wgpu::Renderer::new(
            &mut imgui,
            pixels.device(),
            pixels.queue(),
            imgui_wgpu::RendererConfig {
                texture_format: pixels.render_texture_format(),
                ..Default::default()
            },
        );

        // Return Gui context
        Self {
            imgui,
            platform,
            renderer,
            last_frame: Instant::now(),
            last_cursor: None,
            menubar: Default::default(),
        }
    }

    /// Prepare Dear ImGui.
    pub(crate) fn prepare(
        &mut self,
        window: &winit::window::Window,
    ) -> Result<(), winit::error::ExternalError> {
        // Prepare Dear ImGui
        let now = Instant::now();
        self.imgui.io_mut().update_delta_time(now - self.last_frame);
        self.last_frame = now;
        self.platform.prepare_frame(self.imgui.io_mut(), window)
    }

    pub(crate) fn render(
        &mut self,
        window: &winit::window::Window,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) -> imgui_wgpu::RendererResult<()> {
        // Start a new Dear ImGui frame and update the cursor
        let ui = self.imgui.new_frame();

        let mouse_cursor = ui.mouse_cursor();
        if self.last_cursor != mouse_cursor {
            self.last_cursor = mouse_cursor;
            self.platform.prepare_render(ui, window);
        }

        self.menubar.draw(ui);
        // Draw windows and GUI elements here

        if self.menubar.about.about_open {
            ui.open_popup("About");
            self.menubar.about.about_open = false;
        }

        // Render Dear ImGui with WGPU
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("imgui"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: render_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.renderer.render(
            self.imgui.render(),
            &context.queue,
            &context.device,
            &mut rpass,
        )
    }

    /// Handle any outstanding events
    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<()>,
    ) {
        self.platform
            .handle_event(self.imgui.io_mut(), window, event);
    }

    /// Shows or hides the [Menubar]. If `visible` is [None], toggles visibility.
    pub fn show_menubar(&mut self, visible: Option<bool>) {
        match visible {
            Some(visible) => self.menubar.active = visible,
            None => self.menubar.active ^= true,
        }
    }

    /// Query the state of the Gui through a unified interface
    pub fn wants(&mut self, wants: Wants) -> bool {
        match wants {
            Wants::Quit => self.menubar.file.quit,
            Wants::Disasm => self.menubar.debug.dis,
            Wants::Reset => {
                let reset = self.menubar.debug.reset;
                self.menubar.debug.reset = false;
                reset
            }
            _ => unreachable!(),
        }
    }
}
