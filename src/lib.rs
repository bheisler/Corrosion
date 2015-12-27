pub mod rom;
pub mod memory;
pub mod mappers;
pub mod ppu;
pub mod apu;
pub mod io;

use rom::Rom;
use mappers::Mapper;

use std::rc::Rc;
use std::cell::RefCell;

pub fn start_emulator(rom: Rom) {
    let mapper = Mapper::new(rom.mapper() as u16, rom.prg_rom, rom.chr_rom, rom.prg_ram);
    let mapper: Rc<RefCell<Box<Mapper>>> = Rc::new(RefCell::new(mapper));
    (*mapper).borrow();
}
