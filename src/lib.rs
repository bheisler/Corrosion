pub mod rom;
pub mod memory;
pub mod mappers;
pub mod ppu;
pub mod apu;

use rom::Rom;
use mappers::Mapper;

pub fn start_emulator(rom: Rom) {
    let mapper = Mapper::new(rom.mapper() as u16, rom.prg_rom, rom.chr_rom, rom.prg_ram);
    mapper.borrow();
}
