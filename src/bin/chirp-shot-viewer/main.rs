use chirp::{error::Result, *};
use std::{env::args, fs::read};

fn main() -> Result<()> {
    for screen in args().skip(1).inspect(|screen| println!("{screen}")) {
        let screen = read(screen)?;
        bus! {Screen [0..screen.len()] = &screen}.print_screen()?;
    }
    Ok(())
}
