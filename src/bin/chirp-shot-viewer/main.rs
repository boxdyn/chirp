use chirp::{error::Result, prelude::*};
use std::{env::args, fs::read};

fn main() -> Result<()> {
    bus! {Screen [0..0x100] = &read(args().nth(1).unwrap_or("screen_dump.bin".to_string()))?}
        .print_screen()
}
