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
        match idx {
            0x6000 ... 0x7FFF => self.prg_ram[((idx - 0x6000) as usize % self.prg_ram.len())],
            0x8000 ... 0xFFFF => self.prg_rom[((idx - 0x8000) as usize % self.prg_ram.len())],
            _ => panic!("Invalid NES Memory address"),
        }
    }
    
    fn prg_write(&mut self, idx: u16, val : u8) {
        match idx {
            0x6000 ... 0x7FFF => {
                let idx = (idx - 0x6000) as usize % self.prg_ram.len();
                self.prg_ram[idx] = val;
            }
            0x8000 ... 0xFFFF => (),//Do nothing
            _ => panic!("Invalid NES Memory address"),
        }
    }
    
    fn chr_read(&self, idx: u16) -> u8 {
        self.chr_rom[idx as usize % self.chr_rom.len()]
    }
    
    #[allow(unused_variables)]
    fn chr_write(&mut self, idx: u16, val : u8) {
        //Do Nothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::mappers::Mapper;
    
    #[test]
    fn test_can_create_mapper_0() {
    	NROM::new( vec!(), vec!(), vec!() );
    }
    
    fn create_test_mapper() -> NROM {
        NROM::new( vec!(0u8; 0x4000), vec!(0u8; 0x4000), vec!(0u8; 0x1000) )
    }
    
    #[test]
    fn test_prg_ram_read_write() {
    	let mut nrom = create_test_mapper();
    	println!("{}", nrom.prg_ram.len() );
    	
    	nrom.prg_write( 0x6111, 15 );
    	assert_eq!( nrom.prg_read( 0x6111 ), 15 );
    	
    	nrom.prg_write( 0x6112, 16 );
    	assert_eq!( nrom.prg_read( 0x7112 ), 16 );
    }
    
    #[test]
    fn test_prg_rom_read() {
        let prg_rom : Vec<_> = (0..0x4000)
        	.map(|val| (val % 0xFF) as u8 )
        	.collect();
    	let mapper = NROM::new( prg_rom, vec!(0u8; 0x4000), vec!(0u8; 0x1000) );
    	
    	assert_eq!( mapper.prg_read(0x8111), mapper.prg_read(0xC111));
    }
    
    #[test]
    fn test_prg_rom_write() {
    	let mut mapper = create_test_mapper();
    	
    	mapper.prg_write(0x8612, 15);
    	assert_eq!( mapper.prg_read(0x8612), 0);
    }
    
    #[test]
    fn test_chr_rom_read() {
        let chr_rom : Vec<_> = (0..0x2000)
        	.map(|val| (val % 0xFF) as u8 )
        	.collect();
    	let mapper = NROM::new( vec!(0u8; 0x4000), chr_rom, vec!(0u8; 0x1000) );
    	
    	assert_eq!( mapper.prg_read(0x8111), mapper.prg_read(0xC111));
    }
    
    #[test]
    fn test_chr_rom_write() {
    	let mut mapper = create_test_mapper();
    	
    	mapper.chr_write(0x1612, 15);
    	assert_eq!( mapper.chr_read(0x1612), 0);
    }
}