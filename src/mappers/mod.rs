mod nrom;

use super::memory::MemSegment;
use std::cell::RefCell;

pub trait Mapper {
    fn prg_read(&self, idx: u16) -> u8;
    fn prg_write(&mut self, idx: u16, val: u8);
    fn chr_read(&self, idx: u16) -> u8;
    fn chr_write(&mut self, idx: u16, val: u8);
}

impl Mapper {
    pub fn new(id: u16,
               prg_rom: Vec<u8>,
               chr_rom: Vec<u8>,
               prg_ram: Vec<u8>)
               -> RefCell<Box<Mapper>> {
        match id {
            0 => RefCell::new(Box::new(nrom::NROM::new(prg_rom, chr_rom, prg_ram))),
            _ => panic!("Unsupported Mapper"),
        }
    }
}

impl MemSegment for Mapper {
    fn read(&mut self, idx: u16) -> u8 {
        self.prg_read(idx)
    }
    fn write(&mut self, idx: u16, val: u8) {
        self.prg_write(idx, val)
    }
}
