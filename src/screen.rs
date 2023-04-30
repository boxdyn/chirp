//! Contains the raw screen bytes, and an iterator over individual bits of those bytes.

use crate::traits::Grab;

/// Stores the screen bytes
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Screen {
    /// The screen bytes
    bytes: Vec<u8>,
}

impl<T: AsRef<[u8]>> From<T> for Screen {
    fn from(value: T) -> Self {
        Screen {
            bytes: value.as_ref().into(),
        }
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self::new(256)
    }
}

impl Grab for Screen {
    fn grab<I>(&self, index: I) -> Option<&<I as std::slice::SliceIndex<[u8]>>::Output>
    where
        I: std::slice::SliceIndex<[u8]>,
    {
        self.bytes.get(index)
    }

    fn grab_mut<I>(&mut self, index: I) -> Option<&mut <I as std::slice::SliceIndex<[u8]>>::Output>
    where
        I: std::slice::SliceIndex<[u8]>,
    {
        self.bytes.get_mut(index)
    }
}

impl Screen {
    /// Creates a new [Screen]
    pub fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
        }
    }

    /// Returns true if the screen has 0 elements
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Gets the length of the [Screen]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Gets a slice of the whole [Screen]
    pub fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// Clears the [Screen] to 0
    pub fn clear(&mut self) {
        self.bytes.fill(0);
    }

    /// Grows the [Screen] memory to at least size bytes, but does not truncate
    /// # Examples
    /// ```rust
    ///# use chirp::screen::Screen;
    /// let mut screen = Screen::new(256);
    /// assert_eq!(1234, screen.len());
    /// screen.with_size(0);
    /// assert_eq!(1234, screen.len());
    /// ```
    pub fn with_size(&mut self, size: usize) {
        if self.len() < size {
            self.bytes.resize(size, 0);
        }
    }

    /// Prints the [Screen] using either [drawille] or Box Drawing Characters
    /// # Examples
    /// [Screen::print_screen] will print the screen
    /// ```rust
    ///# use chirp::screen::Screen;
    /// let screen = Screen::default();
    /// screen.print_screen();
    /// ```
    pub fn print_screen(&self) {
        let len_log2 = self.bytes.len().ilog2() / 2;
        #[allow(unused_variables)]
        let (width, height) = (2u32.pow(len_log2 - 1), 2u32.pow(len_log2 + 1) - 1);
        // draw with the drawille library, if available
        #[cfg(feature = "drawille")]
        {
            use drawille::Canvas;
            let mut canvas = Canvas::new(width * 8, height);
            let width = width * 8;
            self.bytes
                .iter()
                .enumerate()
                .flat_map(|(bytei, byte)| {
                    (0..8).enumerate().filter_map(move |(biti, bit)| {
                        if (byte << bit) & 0x80 != 0 {
                            Some(bytei * 8 + biti)
                        } else {
                            None
                        }
                    })
                })
                .for_each(|index| canvas.set(index as u32 % (width), index as u32 / (width)));
            println!("{}", canvas.frame());
        }
        #[cfg(not(feature = "drawille"))]
        for (index, byte) in self.bytes.iter().enumerate() {
            if index % width as usize == 0 {
                print!("{index:03x}|");
            }
            print!(
                "{}",
                format!("{byte:08b}").replace('0', " ").replace('1', "█")
            );
            if index % width as usize == width as usize - 1 {
                println!("|");
            }
        }
    }
}
