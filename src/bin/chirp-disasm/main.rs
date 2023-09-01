use chirp::{error::Result, *};
use gumdrop::*;
use owo_colors::OwoColorize;
use std::{fs::read, path::PathBuf};

mod tree;

fn main() -> Result<()> {
    let mut options = Arguments::parse_args_default_or_exit();
    while let Some(file) = options.file.pop() {
        println!("{file:?}");
        let contents = &read(&file)?;
        if options.tree || options.traverse {
            let loadaddr = options.loadaddr as usize;
            let mem = mem! {
                cpu::mem::Region::Program [loadaddr..loadaddr + contents.len()] = contents
            };
            let mut nodes = Default::default();
            let tree = tree::DisNode::traverse(
                mem.grab(..).expect("grabbing [..] should never fail"),
                &mut nodes,
                options.loadaddr as usize + options.offset,
            );
            if options.traverse {
                for (k, v) in nodes.iter() {
                    if let Some(v) = &v.upgrade().as_ref().map(std::rc::Rc::as_ref) {
                        println!("{k:03x}: {v:04x}");
                    }
                }
            } else {
                println!("{tree}");
            }
        } else {
            let disassembler = Dis::default();
            for (addr, insn) in contents[options.offset..].chunks_exact(2).enumerate() {
                let insn = u16::from_be_bytes(
                    insn.try_into()
                        .expect("Iterated over 2-byte chunks, got <2 bytes"),
                );
                println!(
                    "{:03x}: {:04x} {}",
                    2 * addr + 0x200 + options.offset,
                    insn.bright_black(),
                    disassembler.once(insn),
                );
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Options, Hash)]
struct Arguments {
    #[options(help = "Show help text")]
    help: bool,
    #[options(help = "Load a ROM to run on Chirp", free, required)]
    pub file: Vec<PathBuf>,
    #[options(
        help = "Load address (usually 200)",
        parse(try_from_str = "parse_hex"),
        default = "200"
    )]
    pub loadaddr: u16,
    #[options(help = "Start disassembling at offset...")]
    pub offset: usize,
    #[options(help = "Print the disassembly as a tree")]
    pub tree: bool,
    #[options(help = "Prune unreachable instructions ")]
    pub traverse: bool,
}

fn parse_hex(value: &str) -> std::result::Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(value, 16)
}
