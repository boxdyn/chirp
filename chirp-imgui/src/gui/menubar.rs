//! The menubar that shows at the top of the screen

use super::Drawable;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Menubar {
    pub(super) active: bool,
    pub file: File,
    pub settings: Settings,
    pub debug: Debug,
    pub about: Help,
}

impl Drawable for Menubar {
    fn draw(&mut self, ui: &imgui::Ui) {
        if self.active {
            ui.main_menu_bar(|| {
                self.file.draw(ui);
                self.settings.draw(ui);
                self.debug.draw(ui);
                self.about.draw(ui);
            })
        }
    }
}

impl Default for Menubar {
    fn default() -> Self {
        Self {
            active: true,
            file: Default::default(),
            settings: Default::default(),
            debug: Default::default(),
            about: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct File {
    pub(super) reset: bool,
    pub(super) quit: bool,
}

impl Drawable for File {
    fn draw(&mut self, ui: &imgui::Ui) {
        ui.menu("File", || {
            self.reset = ui.menu_item("Reset");
            self.quit = ui.menu_item("Quit");
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Debug {
    pub(super) reset: bool,
    pub(super) dis: bool,
}

impl Drawable for Debug {
    fn draw(&mut self, ui: &imgui::Ui) {
        ui.menu("Debug", || {
            self.reset = ui.menu_item("Reset");
            ui.checkbox("Live Disassembly", &mut self.dis);
        })
    }
}

impl Debug {
    pub fn reset(&self) -> bool {
        self.reset
    }
    pub fn dis(&self) -> bool {
        self.dis
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Help {
    pub(super) about_open: bool,
}

impl Drawable for Help {
    fn draw(&mut self, ui: &imgui::Ui) {
        ui.menu("Help", || self.about_open = ui.menu_item("About..."))
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Settings {
    pub(super) target_ipf: usize,
    pub(super) quirks: chirp::Quirks,
    pub(super) mode_index: usize,
    pub(super) colors: [[f32; 4]; 2],
    pub(super) applied: bool,
}

impl Drawable for Settings {
    fn draw(&mut self, ui: &imgui::Ui) {
        self.applied = false;
        ui.menu("Settings", || {
            use chirp::Mode::*;
            ui.menu("Foreground Color", || {
                self.applied |= ui.color_picker4("", &mut self.colors[0])
            });
            ui.menu("Background Color", || {
                self.applied |= ui.color_picker4("", &mut self.colors[1])
            });
            const MODES: [chirp::Mode; 3] = [Chip8, SChip, XOChip];
            if ui.combo_simple_string("Mode", &mut self.mode_index, &MODES) {
                self.quirks = MODES[self.mode_index].into();
                self.applied |= true;
            }
            self.applied |= {
                ui.input_scalar("IPF", &mut self.target_ipf)
                    .chars_decimal(true)
                    .build()
                    | ui.checkbox("Bin-ops don't clear vF", &mut self.quirks.bin_ops)
                    | ui.checkbox("DMA doesn't modify I", &mut self.quirks.dma_inc)
                    | ui.checkbox("Draw calls are instant", &mut self.quirks.draw_wait)
                    | ui.checkbox("Screen wraps at edge", &mut self.quirks.screen_wrap)
                    | ui.checkbox("Shift ops ignore vY", &mut self.quirks.shift)
                    | ui.checkbox("Jumps behave eratically", &mut self.quirks.stupid_jumps)
            };
        })
    }
}

impl Settings {
    pub fn target_ipf(&mut self) -> &mut usize {
        &mut self.target_ipf
    }
    pub fn quirks(&mut self) -> &mut chirp::Quirks {
        &mut self.quirks
    }
    pub fn set_mode(&mut self, mode: chirp::Mode) {
        self.mode_index = mode as usize;
    }
    pub fn set_color(&mut self, fg: &[u8; 4], bg: &[u8; 4]) {
        for (idx, component) in fg.iter().enumerate() {
            self.colors[0][idx] = *component as f32 / 255.0;
        }
        for (idx, component) in bg.iter().enumerate() {
            self.colors[1][idx] = *component as f32 / 255.0;
        }
    }
    pub fn applied(&mut self) -> Option<(usize, chirp::Quirks, [u8; 4], [u8; 4])> {
        let (fg, bg) = (self.colors[0], self.colors[1]);
        let fg = [
            (fg[0] * 255.0) as u8,
            (fg[1] * 255.0) as u8,
            (fg[2] * 255.0) as u8,
            (fg[3] * 255.0) as u8,
        ];
        let bg = [
            (bg[0] * 255.0) as u8,
            (bg[1] * 255.0) as u8,
            (bg[2] * 255.0) as u8,
            (bg[3] * 255.0) as u8,
        ];
        self.applied
            .then_some((self.target_ipf, self.quirks, fg, bg))
    }
}
