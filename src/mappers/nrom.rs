use super::Mapper;

pub struct NROM {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,
}

impl NROM {
    pub fn new( prg_rom: Vec<u8>, chr_rom: Vec<u8>, prg_ram: Vec<u8> ) -> NROM {
        NROM {
            prg_rom: prg_rom,
            chr_rom: chr_rom,
            prg_ram: prg_ram,
        }
    }
}

impl Mapper for NROM {
    fn prg_read(&self, idx: u16) -> u8 {
        unimplemented!();
    }
    
    fn prg_write(&mut self, idx: u16, val : u8) {
        unimplemented!();
    }
    
    fn chr_read(&self, idx: u16) -> u8 {
        unimplemented!();
    }
    
    fn chr_write(&mut self, idx: u16, val : u8) {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_can_create_mapper_0() {
    	NROM::new( vec!(), vec!(), vec!() );
    }
}