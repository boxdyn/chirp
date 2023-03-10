//! Dumps data to stdout
use std::ops::Range;

/// Prints a hexdump of a range within the `Dumpable`
///
/// # Examples
/// ```rust
/// # use chumpulator::prelude::*;
/// let mem = Mem::new(0x50);
/// // Dumps the first 0x10 bytes
/// mem.dump(0x00..0x10);
/// ```
pub trait Dumpable {
    /// Prints a hexdump of a range within the object
    fn dump(&self, range: Range<usize>);
}

/// Prints a binary dump of a range within the `Dumpable`
///
/// # Examples
/// ```rust
/// # use chumpulator::prelude::*;
/// let mem = bus! {
///    "mem" [0..0x10] = Mem::new(0x10)
/// };
/// // Dumps the first 0x10 bytes
/// mem.bin_dump(0x00..0x10);
/// ```
pub trait BinDumpable {
    /// Prints a binary dump of a range within the object
    fn bin_dump(&self, _range: Range<usize>) {}
}

pub fn as_hexdump(index: usize, byte: u8) {
    use owo_colors::OwoColorize;
    let term: owo_colors::Style = owo_colors::Style::new().bold().green().on_black();

    if index % 2 == 0 {
        print!(" ")
    }
    if index % 8 == 0 {
        print!(" ")
    }
    if index % 16 == 0 {
        print!("{:>03x}{} ", index.style(term), ":".style(term));
    }
    print!("{byte:02x}");
    if index % 16 == 0xf {
        println!()
    }
}

pub fn as_bindump(index: usize, byte: u8) {
    use owo_colors::OwoColorize;
    let term: owo_colors::Style = owo_colors::Style::new().bold().green().on_black();
    if index % 8 == 0 {
        print!("{:>03x}{} ", index.style(term), ":".style(term));
    }
    print!("{byte:08b} ");
    if index % 8 == 7 {
        println!()
    }
}
