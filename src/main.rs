use chumpulator::{bus::Read, prelude::*};
use std::fs::read;

fn main() -> Result<(), std::io::Error> {
    let mut bus = bus! {
        // Load the charset into ROM
        "charset" [0x0050..0x00a0] = Mem::new(0x50).load_charset(0).w(false),
        // Load the ROM file into RAM
        "userram" [0x0200..0x0F00] = Mem::new(0xF00 - 0x200).load(0, &read("chip-8/Fishie.ch8")?),
        // Create a screen
        "screen"  [0x0F00..0x1000] = Mem::new(32*64/8),
        // Create some stack memory
        "stack"   [0xF000..0xF800] = Mem::new(0x800).r(true).w(true),
    };

    println!("{bus}");

    let disassembler = Disassemble::default();
    for addr in 0x200..0x290 {
        if addr % 2 == 0 {
            println!("{addr:03x}: {}", disassembler.instruction(bus.read(addr)));
        }
    }

    let mut cpu = CPU::new(0xf00, 0x50, 0x200, 0xf7fe, disassembler);
    for _instruction in 0..100 {
        cpu.tick(&mut bus);
        //bus.dump(0xF7e0..0xf800);
    }
    Ok(())
}
