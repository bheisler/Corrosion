#![macro_use]

macro_rules! invalid_address {
    ($e:expr) => (panic!("Invalid NES Memory Address: {:X}", $e));
}

use cart::Cart;
use ppu::PPU;
use apu::APU;
use io::IO;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Range;

pub trait MemSegment {
    fn read(&mut self, idx: u16) -> u8;
    fn read_w(&mut self, idx: u16) -> u16 {
        let low = self.read(idx) as u16;
        let high = self.read(idx + 1) as u16;
        (high << 8) | low
    }

    fn write(&mut self, idx: u16, val: u8);
    fn write_w(&mut self, idx: u16, val: u16) {
        let low = ((val & 0x00FF) >> 0) as u8;
        let high = ((val & 0xFF00) >> 8) as u8;
        self.write(idx, low);
        self.write(idx + 1, high);
    }

    fn print(&mut self, range: Range<u16>) {
        let lower = range.start / 16;
        let upper = range.end / 16 + 1;

        for y in lower..upper {
            print!("{:04X}: ", y * 16);
            for x in 0..16 {
                let addr = (y * 16) + x;
                print!("{:02X} ", self.read(addr));
                if x % 4 == 3 {
                    print!(" ");
                }
            }
            println!("");
        }
    }
}

struct RAM {
    memory: Box<[u8]>,
}

impl RAM {
    fn new() -> RAM {
        RAM { memory: vec!(0u8; 0x0800).into_boxed_slice() }
    }
}

impl MemSegment for RAM {
    fn read(&mut self, idx: u16) -> u8 {
        self.memory[idx as usize % 0x800]
    }

    fn write(&mut self, idx: u16, val: u8) {
        self.memory[idx as usize % 0x800] = val;
    }
}

pub struct CpuMemory {
    ram: RAM,
    pub ppu: PPU,
    pub apu: APU,
    pub io: Box<IO>,
    cart: Rc<RefCell<Cart>>,
}

impl CpuMemory {
    pub fn new(ppu: PPU,
               apu: APU,
               io: Box<IO>,
               cart: Rc<RefCell<Cart>>)
               -> CpuMemory {
        CpuMemory {
            ram: RAM::new(),
            ppu: ppu,
            apu: apu,
            io: io,
            cart: cart,
        }
    }
}

impl MemSegment for CpuMemory {
    fn read(&mut self, idx: u16) -> u8 {
        match idx {
            0x0000...0x1FFF => self.ram.read(idx),
            0x2000...0x3FFF => self.ppu.read(idx),
            0x4000...0x4015 => 0,
            0x4016...0x4017 => self.io.read(idx),
            0x4018...0x4019 => 0,
            0x4020...0xFFFF => self.cart.borrow().prg_read(idx),
            x => invalid_address!(x),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        match idx {
            0x0000...0x1FFF => self.ram.write(idx, val),
            0x2000...0x3FFF => self.ppu.write(idx, val),
            0x4000...0x4015 => self.apu.write(idx, val),
            0x4016 => self.io.write(idx, val),
            0x4017...0x4019 => self.apu.write(idx, val),
            0x4020...0xFFFF => self.cart.borrow_mut().prg_write(idx, val),
            x => invalid_address!(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;
    use mappers::{Mapper, MapperParams};
    use screen::DummyScreen;
    use io::DummyIO;
    use audio::DummyAudioOut;

    fn create_test_memory() -> CpuMemory {
        let nrom = Mapper::new(0,
                               MapperParams::simple(vec!(0u8; 0x4000), vec!(0u8; 0x4000)));
        let cart = ::cart::Cart::new(nrom);
        let cart = Rc::new(RefCell::new(cart));
        let ppu = ::ppu::PPU::new(cart.clone(), Box::new(DummyScreen::new()));
        let apu = ::apu::APU::new(Box::new(DummyAudioOut));
        let io = DummyIO::new();
        CpuMemory::new(ppu, apu, Box::new(io), cart)
    }

    #[test]
    fn can_read_write_ram_through_memory() {
        let mut mem = create_test_memory();

        mem.write(0x0000, 0x24);
        assert_eq!(mem.read(0x0000), 0x24);

        mem.write(0x0799, 0x25);
        assert_eq!(mem.read(0x0799), 0x25);
    }

    #[test]
    fn test_ram_mirroring() {
        let mut mem = create_test_memory();

        mem.write(0x0800, 12);
        assert_eq!(mem.read(0x0000), 12);

        mem.write(0x1952, 12);
        assert_eq!(mem.read(0x0152), 12);
    }

    #[test]
    fn can_read_write_prg_ram_through_memory() {
        let mut mem = create_test_memory();

        mem.write(0x6111, 0x24);
        assert_eq!(mem.read(0x6111), 0x24);

        mem.write(0x6799, 0x25);
        assert_eq!(mem.read(0x6799), 0x25);
    }

    #[test]
    fn can_read_write_to_ppu_registers_through_memory() {
        let mut mem = create_test_memory();

        // We're relying on the PPU dynamic latch effect to get the right answers
        mem.write(0x2000, 0x45);
        assert_eq!(mem.ppu.read(0x2000), 0x45);

        mem.write(0x2000, 0x48);
        assert_eq!(mem.read(0x2000), 0x48);
    }

    #[test]
    fn test_read_w_reads_low_byte_first() {
        let mut mem = create_test_memory();

        mem.write(0x1000, 0xCD);
        mem.write(0x1001, 0xAB);

        assert_eq!(mem.read_w(0x1000), 0xABCD);
    }
}
