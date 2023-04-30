// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! The Mem represents the CPU's memory
//!
//! Contains some handy utils for reading and writing

use crate::{error::Result, traits::Grab};
use std::{
    fmt::{Debug, Display, Formatter},
    ops::Range,
    slice::SliceIndex,
};

/// Creates a new [Mem], growing as needed
/// # Examples
/// ```rust
/// # use chirp::*;
/// let mut mem = mem! {
///     Charset   [0x0000..0x0800] = b"ABCDEF",
///     Program [0x0800..0xf000] = include_bytes!("mem.rs"),
/// };
/// ```
#[macro_export]
macro_rules! mem {
    ($($name:path $(:)? [$range:expr] $(= $data:expr)?) ,* $(,)?) => {
        $crate::cpu::mem::Mem::default()$(.add_region_owned($name, $range)$(.load_region_owned($name, $data))?)*
    };
}

// Traits Read and Write are here purely to make implementing other things more bearable
impl Grab for Mem {
    /// Gets a slice of [Mem] memory
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mem = Mem::new()
    ///         .add_region_owned(Program, 0..10);
    ///     assert!([0;10].as_slice() == mem.grab(0..10).unwrap());
    ///#    Ok(())
    ///# }
    /// ```
    #[inline(always)]
    fn grab<I>(&self, index: I) -> Option<&<I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.memory.get(index)
    }

    /// Gets a mutable slice of [Mem] memory
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mut mem = Mem::new()
    ///         .add_region_owned(Program, 0..10);
    ///     assert!([0;10].as_slice() == mem.grab_mut(0..10).unwrap());
    ///#    Ok(())
    ///# }
    /// ```
    #[inline(always)]
    fn grab_mut<I>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.memory.get_mut(index)
    }
}

/// Represents a named region in memory
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Region {
    /// Character ROM (but writable!)
    Charset,
    /// Program memory
    Program,
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
                _ => "",
            }
        )
    }
}

/// Stores memory in a series of named regions with ranges
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mem {
    memory: Vec<u8>,
    region: [Option<Range<usize>>; Region::Count as usize],
}

impl Mem {
    // TODO: make mem::new() give a properly set up mem with a default memory map
    /// Constructs a new mem
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mem = Mem::new();
    ///     assert!(mem.is_empty());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn new() -> Self {
        Mem::default()
    }

    /// Gets the length of the mem' backing memory
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mem = Mem::new()
    ///         .add_region_owned(Program, 0..1234);
    ///     assert_eq!(1234, mem.len());
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
    ///     let mem = Mem::new();
    ///     assert!(mem.is_empty());
    ///#    Ok(())
    ///# }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.memory.is_empty()
    }

    /// Grows the Mem backing memory to at least size bytes, but does not truncate
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mut mem = Mem::new();
    ///     mem.with_size(1234);
    ///     assert_eq!(1234, mem.len());
    ///     mem.with_size(0);
    ///     assert_eq!(1234, mem.len());
    ///#    Ok(())
    ///# }
    /// ```
    fn with_size(&mut self, size: usize) {
        if self.len() < size {
            self.memory.resize(size, 0);
        }
    }

    /// Adds a new names range ([Region]) to an owned [Mem]
    pub fn add_region_owned(mut self, name: Region, range: Range<usize>) -> Self {
        self.add_region(name, range);
        self
    }

    /// Adds a new named range ([Region]) to a [Mem]
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mut mem = Mem::new();
    ///     mem.add_region(Program, 0..1234);
    ///     assert_eq!(1234, mem.len());
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
    ///     let mut mem = Mem::new().add_region_owned(Program, 0..1234);
    ///     mem.set_region(Program, 1234..2345);
    ///     assert_eq!(2345, mem.len());
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

    /// Loads data into a [Region] on an *owned* [Mem], for use during initialization
    pub fn load_region_owned(mut self, name: Region, data: &[u8]) -> Self {
        self.load_region(name, data).ok();
        self
    }

    /// Loads data into a named [Region]
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mem = Mem::new()
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
    ///     let mem = Mem::new()
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
    ///     let mem = Mem::new()
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
    ///     let mem = Mem::new()
    ///         .add_region_owned(Program, 0..10);
    ///     assert!([0;10].as_slice() == mem.get_region(Program).unwrap());
    ///#    Ok(())
    ///# }
    /// ```
    #[inline(always)]
    pub fn get_region(&self, name: Region) -> Option<&[u8]> {
        debug_assert!(self.region.get(name as usize).is_some());
        self.grab(self.region.get(name as usize)?.clone()?)
    }

    /// Gets a mutable slice of a named region of memory
    /// # Examples
    /// ```rust
    ///# use chirp::*;
    ///# fn main() -> Result<()> {
    ///     let mut mem = Mem::new()
    ///         .add_region_owned(Program, 0..10);
    ///     assert!([0;10].as_slice() == mem.get_region_mut(Program).unwrap());
    ///#    Ok(())
    ///# }
    /// ```
    #[inline(always)]
    pub fn get_region_mut(&mut self, name: Region) -> Option<&mut [u8]> {
        debug_assert!(self.region.get(name as usize).is_some());
        self.grab_mut(self.region.get(name as usize)?.clone()?)
    }
}

#[cfg(target_feature = "rhexdump")]
impl Display for Mem {
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
