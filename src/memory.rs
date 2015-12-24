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
        self.memory[idx as usize]
    }
    
    fn write(&mut self, idx: u16, val: u8) {
        self.memory[idx as usize] = val;
    }
}

pub struct CpuMemory {
    ram: RAM,
    //ppu: &MemSegment,
    //apu: &MemSegment,
    //cart: &MemSegment,
}

impl CpuMemory {
    pub fn new( ) -> CpuMemory {
        CpuMemory{
            ram: RAM::new(),
        }
    }
}

impl MemSegment for CpuMemory {
    fn read(&self, idx:u16) -> u8 {
        match idx {
            0x0000 ... 0x1FFF => self.ram.read( idx % 0x0800 ),
            _ => panic!( "Invalid NES memory address!" )
        }
    }
    
    fn write(&mut self, idx:u16, val:u8) {
        match idx {
            0x0000 ... 0x1FFF => self.ram.write( idx % 0x0800, val ),
            _ => panic!( "Invalid NES memory address!" )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn can_read_write_ram_through_memory() {
    	let mut mem = CpuMemory::new();
    	
    	mem.write( 0x0000, 0x24 );
    	assert_eq!( mem.read( 0x0000 ), 0x24 );
    	
    	mem.write( 0x0799, 0x25 );
    	assert_eq!( mem.read( 0x0799 ), 0x25 );
    }
    
    #[test]
    fn test_ram_mirroring() {
    	let mut mem = CpuMemory::new();
    	
    	mem.write(0x0800, 12);
    	assert_eq!( mem.read( 0x0000 ), 12 );
    	    
	    mem.write(0x1952, 12);
    	assert_eq!( mem.read( 0x0152 ), 12 );
    }
}