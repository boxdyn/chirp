use chirp::{cpu::Disassembler, error::Result, *};
use gumdrop::*;
use owo_colors::OwoColorize;
use std::{fs::read, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Options, Hash)]
struct Arguments {
    #[options(help = "Show help text")]
    help: bool,
    #[options(help = "Load a ROM to run on Chirp", free, required)]
    pub file: PathBuf,
    #[options(help = "Load address (usually 200)", parse(try_from_str = "parse_hex"))]
    pub loadaddr: u16,
    #[options(help = "Start disassembling at offset...")]
    pub offset: usize,
}

fn parse_hex(value: &str) -> std::result::Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(value, 16)
}

fn main() -> Result<()> {
    let options = Arguments::parse_args_default_or_exit();
    let contents = &read(&options.file)?;
    let disassembler = Dis::default();
    for (addr, insn) in contents[options.offset..].chunks_exact(2).enumerate() {
        let insn = u16::from_be_bytes(
            insn.try_into()
                .expect("Iterated over 2-byte chunks, got <2 bytes"),
        );
        println!(
            "{}",
            format_args!(
                "{:03x}: {} {:04x}",
                2 * addr + 0x200 + options.offset,
                disassembler.once(insn),
                insn.bright_black(),
            )
        );
    }
    Ok(())
}
