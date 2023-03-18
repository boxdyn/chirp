use chumpulator::{bus::Read, prelude::*};
use std::fs::read;
use std::time::{Duration, Instant};

/// What I want:
/// I want a data bus that stores as much memory as I need to implement a chip 8 emulator
/// I want that data bus to hold named memory ranges and have a way to get a memory region

fn main() -> Result<(), std::io::Error> {
    let mut now;
    println!("Building Bus...");
    let mut time = Instant::now();
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
    now = time.elapsed();
    println!("Elapsed: {:?}\nBuilding NewBus...", now);
    time = Instant::now();
    let mut newbus = newbus! {
        // Load the charset into ROM
        "charset" [0x0050..0x00a0] = include_bytes!("mem/charset.bin"),
        // Load the ROM file into RAM
        "userram" [0x0200..0x0F00] = &read("chip-8/Fishie.ch8")?,
        // Create a screen
        "screen"  [0x0F00..0x1000],
        // Create some stack memory
        "stack"   [0x2000..0x2800],
    };
    now = time.elapsed();
    println!("Elapsed: {:?}", now);
    println!("{newbus}");

    let disassembler = Disassemble::default();
    if false {
        for addr in 0x200..0x290 {
            if addr % 2 == 0 {
                println!(
                    "{addr:03x}: {}",
                    disassembler.instruction(bus.read(addr as usize))
                );
            }
        }
    }

    let mut cpu = CPU::new(0xf00, 0x50, 0x200, 0xf7fe, disassembler);
    let mut cpu2 = cpu.clone();
    println!("Old Bus:");
    for _instruction in 0..6 {
        time = Instant::now();
        cpu.tick(&mut bus);
        now = time.elapsed();
        println!("         Elapsed: {:?}", now);
        std::thread::sleep(Duration::from_micros(2000).saturating_sub(time.elapsed()));
    }
    println!("New Bus:");
    for _instruction in 0..6 {
        time = Instant::now();
        cpu2.tick(&mut newbus);
        now = time.elapsed();
        println!("         Elapsed: {:?}", now);
        std::thread::sleep(Duration::from_micros(2000).saturating_sub(time.elapsed()));
    }
    Ok(())
}
