//! Chirp: A chip-8 interpreter in Rust
//! Hello, world!

use chirp::{error::Result, prelude::*};
use gumdrop::*;
use minifb::*;
use std::fs::read;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Options, Hash)]
struct Arguments {
    #[options(help = "Enable behavior incompatible with modern software")]
    pub authentic: bool,
    #[options(
        help = "Set breakpoints for the emulator to stop at",
        parse(try_from_str = "parse_hex")
    )]
    pub breakpoints: Vec<u16>,
    #[options(help = "Enable debug mode at startup")]
    pub debug: bool,
    #[options(help = "Enable pause mode at startup")]
    pub pause: bool,
    #[options(help = "Load a ROM to run on Chirp")]
    pub file: PathBuf,
}

fn parse_hex(value: &str) -> std::result::Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(value, 16)
}

fn main() -> Result<()> {
    let options = Arguments::parse_args_default_or_exit();

    // Create the data bus
    let mut bus = bus! {
        // Load the charset into ROM
        "charset" [0x0050..0x00A0] = include_bytes!("mem/charset.bin"),
        // Load the ROM file into RAM
        "userram" [0x0200..0x0F00] = &read(options.file)?,
        // Create a screen
        "screen"  [0x0F00..0x1000],
        // Create some stack memory
        //"stack"   [0x2000..0x2100],
    };

    // let disassembler = Disassemble::default();
    // if false {
    //     for addr in 0x200..0x290 {
    //         if addr % 2 == 0 {
    //             println!(
    //                 "{addr:03x}: {}",
    //                 disassembler.instruction(bus.read(addr as usize))
    //             );
    //         }
    //     }
    // }

    let mut cpu = CPU::new(0xf00, 0x50, 0x200, 0x20fe, Disassemble::default());
    for point in options.breakpoints {
        cpu.set_break(point);
    }
    cpu.flags.authentic = options.authentic;
    cpu.flags.debug = options.debug;
    cpu.flags.pause = options.pause;
    let mut framebuffer = FrameBuffer::new(64, 32);
    let mut window = WindowBuilder::default().build()?;
    let mut frame_time = Instant::now();
    let mut step_time = Instant::now();

    framebuffer.render(&mut window, &mut bus);

    cpu.flags.pause = false;
    cpu.flags.debug = true;

    loop {
        if !cpu.flags.pause {
            cpu.tick(&mut bus);
        }
        while frame_time.elapsed() > Duration::from_micros(16000) {
            if cpu.flags.pause {
                window.set_title("Chirp  ⏸")
            } else {
                window.set_title("Chirp  ▶")
            }
            frame_time += Duration::from_micros(16000);
            // tick sound and delay timers
            cpu.tick_timer();
            // update framebuffer
            framebuffer.render(&mut window, &mut bus);
            // get key input (has to happen after framebuffer)
            get_keys(&mut window, &mut cpu);
            // handle keys at the
            for key in window.get_keys_pressed(KeyRepeat::No) {
                use Key::*;
                match key {
                    F1 => cpu.dump(),
                    F2 => bus
                        .print_screen()
                        .expect("The 'screen' memory region exists"),
                    F3 => {
                        println!(
                            "{}",
                            endis("Debug", {
                                cpu.flags.debug();
                                cpu.flags.debug
                            })
                        )
                    }
                    F4 => println!(
                        "{}",
                        endis("Pause", {
                            cpu.flags.pause();
                            cpu.flags.pause
                        })
                    ),
                    F5 => {
                        println!("Step");
                        cpu.singlestep(&mut bus)
                    }
                    F6 => {
                        println!("Set breakpoint {:x}", cpu.pc());
                        cpu.set_break(cpu.pc())
                    }
                    F7 => {
                        println!("Unset breakpoint {:x}", cpu.pc());
                        cpu.unset_break(cpu.pc())
                    }
                    F8 => {
                        println!("Soft reset CPU {:x}", cpu.pc());
                        cpu.soft_reset();
                        bus.clear_region("screen");
                    }
                    F9 => {
                        println!("Hard reset CPU");
                        cpu = CPU::default();
                        bus.clear_region("screen");
                    }
                    Escape => return Ok(()),
                    _ => (),
                }
            }
        }
        std::thread::sleep(Duration::from_micros(1666).saturating_sub(step_time.elapsed()));
        step_time = Instant::now();
    }
    //Ok(())
}

fn endis(name: &str, state: bool) -> String {
    format!("{name} {}", if state { "enabled" } else { "disabled" })
}
