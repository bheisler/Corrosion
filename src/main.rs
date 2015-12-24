mod rom;
mod memory;
mod mappers;

use std::env;
use std::path::Path;

fn main() {
    let args = env::args();
    let rom_name = args.skip(1).next().expect("No ROM file provided.");
    let path = Path::new(&rom_name);
    let mut rom = rom::Rom::read(&path).expect("Failed to read ROM File");
    let mapper = mappers::Mapper::new( rom.mapper() as u16, rom.prg_rom, rom.chr_rom, rom.prg_ram );
}
