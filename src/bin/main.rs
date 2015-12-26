extern crate nes_emulator;

use std::env;
use std::path::Path;
use nes_emulator::rom::Rom;
use nes_emulator::start_emulator;

fn main() {
    let args = env::args();
    let rom_name = args.skip(1).next().expect("No ROM file provided.");
    let path = Path::new(&rom_name);
    let rom = Rom::read(&path).expect("Failed to read ROM File");
    start_emulator(rom);
}
