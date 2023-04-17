// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE.txt for details)

//! The Bus connects the CPU to Memory
//!
//! This is more of a memory management unit + some utils for reading/writing

use crate::error::{Error::MissingRegion, Result};
use std::{
    fmt::{Debug, Display, Formatter},
    ops::Range,
    slice::SliceIndex,
};

/// Creates a new bus, growing the backing memory as needed
/// # Examples
/// ```rust
/// # use chirp::*;
/// let mut bus = bus! {
///     Stack   [0x0000..0x0800] = b"ABCDEF",
///     Program [0x0800..0xf000] = include_bytes!("bus.rs"),
/// };
/// ```
#[macro_export]
macro_rules! bus {
    ($($name:path $(:)? [$range:expr] $(= $data:expr)?) ,* $(,)?) => {
        $crate::cpu::bus::Bus::default()
        $(
            .add_region_owned($name, $range)
            $(
                .load_region_owned($name, $data)
            )?
        )*
    };
}

pub mod read;
pub use read::{Get, ReadWrite};

// Traits Read and Write are here purely to make implementing other things more bearable
impl Get<u8> for Bus {
    /// Gets a slice of [Bus] memory
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let bus = Bus::new()
    ///         .add_region_owned(Program, 0..10);
    ///     assert!([0;10].as_slice() == bus.get(0..10).unwrap());
    ///#    Ok(())
    ///# }
    /// ```
    #[inline(always)]
    fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.memory.get(index)
    }

    /// Gets a mutable slice of [Bus] memory
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mut bus = Bus::new()
    ///         .add_region_owned(Program, 0..10);
    ///     assert!([0;10].as_slice() == bus.get_mut(0..10).unwrap());
    ///#    Ok(())
    ///# }
    /// ```
    #[inline(always)]
    fn get_mut<I>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.memory.get_mut(index)
    }
}

/// Represents a named region in memory
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Region {
    /// Character ROM (but writable!)
    Charset,
    /// Program memory
    Program,
    /// Screen buffer
    Screen,
    /// Stack space
    Stack,
    #[doc(hidden)]
    /// Total number of named regions
    Count,
}

impl Display for Region {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Region::Charset => "Charset",
                Region::Program => "Program",
                Region::Screen => "Screen",
                Region::Stack => "Stack",
                _ => "",
            }
        )
    }
}

/// Stores memory in a series of named regions with ranges
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bus {
    memory: Vec<u8>,
    region: [Option<Range<usize>>; Region::Count as usize],
}

impl Bus {
    // TODO: make bus::new() give a properly set up bus with a default memory map
    /// Constructs a new bus
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let bus = Bus::new();
    ///     assert!(bus.is_empty());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn new() -> Self {
        Bus::default()
    }

    /// Gets the length of the bus' backing memory
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let bus = Bus::new()
    ///         .add_region_owned(Program, 0..1234);
    ///     assert_eq!(1234, bus.len());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn len(&self) -> usize {
        self.memory.len()
    }

    /// Returns true if the backing memory contains no elements
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let bus = Bus::new();
    ///     assert!(bus.is_empty());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.memory.is_empty()
    }
    /// Grows the Bus backing memory to at least size bytes, but does not truncate
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mut bus = Bus::new();
    ///     bus.with_size(1234);
    ///     assert_eq!(1234, bus.len());
    ///     bus.with_size(0);
    ///     assert_eq!(1234, bus.len());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn with_size(&mut self, size: usize) {
        if self.len() < size {
            self.memory.resize(size, 0);
        }
    }

    /// Adds a new names range ([Region]) to an owned [Bus]
    pub fn add_region_owned(mut self, name: Region, range: Range<usize>) -> Self {
        self.add_region(name, range);
        self
    }

    /// Adds a new named range ([Region]) to a [Bus]
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mut bus = Bus::new();
    ///     bus.add_region(Program, 0..1234);
    ///     assert_eq!(1234, bus.len());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn add_region(&mut self, name: Region, range: Range<usize>) -> &mut Self {
        self.with_size(range.end);
        if let Some(region) = self.region.get_mut(name as usize) {
            *region = Some(range);
        }
        self
    }

    /// Updates an existing [Region]
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mut bus = Bus::new().add_region_owned(Program, 0..1234);
    ///     bus.set_region(Program, 1234..2345);
    ///     assert_eq!(2345, bus.len());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn set_region(&mut self, name: Region, range: Range<usize>) -> &mut Self {
        self.with_size(range.end);
        if let Some(region) = self.region.get_mut(name as usize) {
            *region = Some(range);
        }
        self
    }

    /// Loads data into a [Region] on an *owned* [Bus], for use during initialization
    pub fn load_region_owned(mut self, name: Region, data: &[u8]) -> Self {
        self.load_region(name, data).ok();
        self
    }

    /// Loads data into a named [Region]
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let bus = Bus::new()
    ///         .add_region_owned(Program, 0..1234)
    ///         .load_region(Program, b"Hello, world!")?;
    ///# // TODO: Test if region actually contains "Hello, world!"
    ///#    Ok(())
    ///# }
    /// ```
    pub fn load_region(&mut self, name: Region, data: &[u8]) -> Result<&mut Self> {
        use std::io::Write;
        if let Some(mut region) = self.get_region_mut(name) {
            assert_eq!(region.write(data)?, data.len());
        }
        Ok(self)
    }

    /// Fills a [Region] with zeroes
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let bus = Bus::new()
    ///         .add_region_owned(Program, 0..1234)
    ///         .clear_region(Program);
    ///# // TODO: test if region actually clear
    ///#    Ok(())
    ///# }
    /// ```
    /// If the region doesn't exist, that's okay.
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let bus = Bus::new()
    ///         .add_region_owned(Program, 0..1234)
    ///         .clear_region(Screen);
    ///# // TODO: test if region actually clear
    ///#    Ok(())
    ///# }
    /// ```
    pub fn clear_region(&mut self, name: Region) -> &mut Self {
        if let Some(region) = self.get_region_mut(name) {
            region.fill(0)
        }
        self
    }

    /// Gets a slice of a named [Region] of memory
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let bus = Bus::new()
    ///         .add_region_owned(Program, 0..10);
    ///     assert!([0;10].as_slice() == bus.get_region(Program).unwrap());
    ///#    Ok(())
    ///# }
    /// ```
    #[inline(always)]
    pub fn get_region(&self, name: Region) -> Option<&[u8]> {
        debug_assert!(self.region.get(name as usize).is_some());
        self.get(self.region.get(name as usize)?.clone()?)
    }

    /// Gets a mutable slice of a named region of memory
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mut bus = Bus::new()
    ///         .add_region_owned(Program, 0..10);
    ///     assert!([0;10].as_slice() == bus.get_region_mut(Program).unwrap());
    ///#    Ok(())
    ///# }
    /// ```
    #[inline(always)]
    pub fn get_region_mut(&mut self, name: Region) -> Option<&mut [u8]> {
        debug_assert!(self.region.get(name as usize).is_some());
        self.get_mut(self.region.get(name as usize)?.clone()?)
    }

    /// Prints the region of memory called `Screen` at 1bpp using box characters
    /// # Examples
    ///
    /// [Bus::print_screen] will print the screen
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let bus = Bus::new()
    ///         .add_region_owned(Screen, 0x000..0x100);
    ///     bus.print_screen()?;
    ///#    Ok(())
    ///# }
    /// ```
    /// If there is no Screen region, it will return Err([MissingRegion])
    /// ```rust,should_panic
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mut bus = Bus::new()
    ///         .add_region_owned(Program, 0..10);
    ///     bus.print_screen()?;
    ///#    Ok(())
    ///# }
    /// ```
    pub fn print_screen(&self) -> Result<()> {
        const REGION: Region = Region::Screen;
        if let Some(screen) = self.get_region(REGION) {
            let len_log2 = screen.len().ilog2() / 2;
            #[allow(unused_variables)]
            let (width, height) = (2u32.pow(len_log2 - 1), 2u32.pow(len_log2 + 1) - 1);
            // draw with the drawille library, if available
            #[cfg(feature = "drawille")]
            {
                use drawille::Canvas;
                let mut canvas = Canvas::new(dbg!(width * 8), dbg!(height));
                let width = width * 8;
                screen
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
            for (index, byte) in screen.iter().enumerate() {
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
        } else {
            return Err(MissingRegion { region: REGION });
        }
        Ok(())
    }
}

#[cfg(target_feature = "rhexdump")]
impl Display for Bus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use rhexdump::Rhexdump;
        let mut rhx = Rhexdump::default();
        rhx.set_bytes_per_group(2)
            .expect("2 <= MAX_BYTES_PER_GROUP (8)");
        rhx.display_duplicate_lines(false);
        for (&name, range) in &self.region {
            writeln!(
                f,
                "[{name}]\n{}\n",
                rhx.hexdump(&self.memory[range.clone()])
            )?
        }
        write!(f, "")
    }
}
