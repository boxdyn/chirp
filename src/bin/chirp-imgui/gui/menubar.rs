//! The menubar that shows at the top of the screen

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Menubar {
    pub(super) active: bool,
    pub file: File,
    pub settings: Settings,
    pub debug: Debug,
    pub about: About,
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
    pub(super) settings: bool,
    pub(super) reset: bool,
    pub(super) quit: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Debug {
    pub(super) reset: bool,
    pub(super) dis: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct About {
    pub(super) open: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Settings {
    pub(super) target_ipf: usize,
    pub(super) quirks: chirp::Quirks,
    pub(super) mode_index: usize,
    pub(super) colors: [[u8; 4]; 2],
    pub(super) applied: bool,
}

impl Settings {
    pub fn target_ipf(&mut self) -> &mut usize {
        &mut self.target_ipf
    }
    pub fn quirks(&mut self) -> &mut chirp::Quirks {
        &mut self.quirks
    }

    pub fn applied(&mut self) -> Option<(usize, chirp::Quirks)> {
        self.applied.then_some((self.target_ipf, self.quirks))
    }
}
