#![macro_use]

macro_rules! invalid_address {
    ($e:expr) => (panic!("Invalid NES Memory Address: {:X}", $e));
}

use mappers::Mapper;
use std::cell::RefCell;

pub trait MemSegment {
    fn read(&self, idx: u16) -> u8;
    fn write(&mut self, idx: u16, val: u8);
}

struct RAM {
    memory: Box<[u8]>,
}

impl RAM {
    fn new() -> RAM {
        RAM{ memory: vec!(0u8; 0x07ff).into_boxed_slice() }
    }
}

impl MemSegment for RAM {
    fn read(&self, idx: u16) -> u8 {
        self.memory[idx as usize % 0x800]
    }
    
    fn write(&mut self, idx: u16, val: u8) {
        self.memory[idx as usize% 0x800] = val;
    }
}

pub struct CpuMemory {
    ram: RAM,
    //ppu: &MemSegment,
    //apu: &MemSegment,
    //input
    cart: RefCell<Box<Mapper>>,
}

impl CpuMemory {
    pub fn new( cart: RefCell<Box<Mapper>> ) -> CpuMemory {
        CpuMemory{
            ram: RAM::new(),
            cart: cart,
        }
    }
}

impl MemSegment for CpuMemory {
    fn read(&self, idx:u16) -> u8 {
        match idx {
            0x0000 ... 0x1FFF => self.ram.read( idx ),
            0x4020 ... 0xFFFF => self.cart.borrow().prg_read( idx ),
            x => invalid_address!(x),
        }
    }
    
    fn write(&mut self, idx:u16, val:u8) {
        match idx {
            0x0000 ... 0x1FFF => self.ram.write( idx, val ),
            0x4020 ... 0xFFFF => self.cart.borrow_mut().prg_write( idx, val ),
            x => invalid_address!(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_memory() -> CpuMemory {
        let nrom = ::mappers::Mapper::new( 0, vec!(0u8; 0x4000), vec!(0u8; 0x4000), vec!(0u8; 0x1000));
        CpuMemory::new(nrom)
    }
    
    #[test]
    fn can_read_write_ram_through_memory() {
        let mut mem = create_test_memory();
        
        mem.write( 0x0000, 0x24 );
        assert_eq!( mem.read( 0x0000 ), 0x24 );
        
        mem.write( 0x0799, 0x25 );
        assert_eq!( mem.read( 0x0799 ), 0x25 );
    }
    
    #[test]
    fn test_ram_mirroring() {
        let mut mem = create_test_memory();
        
        mem.write(0x0800, 12);
        assert_eq!( mem.read( 0x0000 ), 12 );
            
        mem.write(0x1952, 12);
        assert_eq!( mem.read( 0x0152 ), 12 );
    }
    
    #[test]
    fn can_read_write_prg_ram_through_memory() {
        let mut mem = create_test_memory();
        
        mem.write( 0x6111, 0x24 );
        assert_eq!( mem.read( 0x6111 ), 0x24 );
        
        mem.write( 0x6799, 0x25 );
        assert_eq!( mem.read( 0x6799 ), 0x25 );
    }
}