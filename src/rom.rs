const MAGIC_NUMBERS : [u8; 4] = [0x4Eu8, 0x45u8, 0x53u8, 0x1Au8];
pub const PRG_ROM_PAGE_SIZE : usize = 16384;
pub const CHR_ROM_PAGE_SIZE : usize = 8192;
pub const PRG_RAM_PAGE_SIZE : usize = 8192;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ScreenMode {
	Horizontal,
	Vertical,
	FourScreen
}

pub struct Rom {
	flags6: u8,
	flags7: u8,
	prg_rom: Vec<u8>,
	chr_rom: Vec<u8>,
	prg_ram: Vec<u8>,
	trainer: Vec<u8>,
}

fn get_bit(byte: u8, bit_num: u8) -> bool {
	!((byte & 1u8 << bit_num) == 0)
}

fn read_byte( iter : &mut Iterator<Item=u8> ) -> u8 {
	iter.next().unwrap()
}

fn read_bytes( iter: &mut Iterator<Item=u8>, bytes: usize ) -> Vec<u8> {
	iter.take( bytes ).collect()
}

impl Rom {
	///Parse the given bytes as an iNES 1.0 header.
	///NES 2.0 is not supported until I can find a rom that actually uses it.
	pub fn parse(data: &Vec<u8>) -> Rom {
		let mut iter = data.iter().cloned();
		assert_eq!( read_bytes( &mut iter, 4 ), MAGIC_NUMBERS );
		let prg_rom_pages = read_byte( &mut iter );
		let chr_rom_pages = read_byte( &mut iter );
		let flags6 = read_byte( &mut iter );
		let flags7 = read_byte( &mut iter );
		let prg_ram_pages = match read_byte( &mut iter ) {
			0 => 1,
			x => x,
		};
		
		read_bytes( &mut iter, 7 );
		
		let has_trainer = get_bit(flags6, 2);
		let trainer = match has_trainer {
			false => vec!(),
			true => read_bytes( &mut iter, 512 ),
		};
		
		Rom { 
			prg_rom: read_bytes( &mut iter, PRG_ROM_PAGE_SIZE * prg_rom_pages as usize ),
			chr_rom: read_bytes( &mut iter, CHR_ROM_PAGE_SIZE * chr_rom_pages as usize ),
			flags6: flags6,
			flags7: flags7,
			prg_ram: vec!( 0u8; PRG_RAM_PAGE_SIZE * prg_ram_pages as usize ),
			trainer: trainer
		}
	}
	
	pub fn screen_mode(&self) -> ScreenMode {
		match self.flags6 & 0b0000_1001u8 {
			0b0000_0000 => ScreenMode::Horizontal,
			0b0000_0001 => ScreenMode::Vertical,
			0b0000_1000 | 0b0000_1001 => ScreenMode::FourScreen,
			_ => panic!("Math is broken!"),
		}
	}
	
	pub fn sram(&self) -> bool {
		get_bit(self.flags6, 1)
	}
	
	pub fn trainer(&self) -> &Vec<u8> {
		&self.trainer
	}
	
	pub fn pc10(&self) -> bool {
		get_bit(self.flags7, 0)
	}
	
	pub fn vs(&self) -> bool {
		get_bit(self.flags7, 1)
	}
	
	pub fn mapper(&self) -> u8 {
		( ( self.flags6 & 0xF0 ) >> 4 ) | ( self.flags7 & 0xF0 )
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rand::{Rng, thread_rng};
	
	struct RomBuilder {
		header : Vec<u8>,
		trainer : Vec<u8>,
		prg_rom : Vec<u8>,
		chr_rom : Vec<u8>,
	}
	
	fn set_bit(byte: &mut u8, bit_num: u8) {
		*byte = *byte | 1u8 << bit_num;
	}
	
	fn generate_bytes( size: usize ) -> Vec<u8> {
		let mut rng = thread_rng();
		let mut bytes : Vec<u8> = vec!(0u8; size);
		rng.fill_bytes(&mut bytes);
		bytes
	}
	
	impl RomBuilder {
		fn new() -> RomBuilder {
			let mut header = vec![0x4Eu8, 0x45u8, 0x53u8, 0x1Au8];
			header.extend([0; 12].iter().cloned());
			return RomBuilder{ 
				header: header, 
				trainer: vec!(),
				prg_rom: vec!(),
				chr_rom: vec!(),
			}
		}
		
		fn set_prg_page_count( &mut self, count: u8 ) {
			self.header[4] = count;
			self.prg_rom = generate_bytes( count as usize * PRG_ROM_PAGE_SIZE );
		}
		
		fn set_chr_page_count( &mut self, count: u8 ) {
			self.header[5] = count;
			self.chr_rom = generate_bytes( count as usize * CHR_ROM_PAGE_SIZE );
		}
		
		fn set_mirroring( &mut self ) {
			set_bit(&mut self.header[6], 0)
		}
		
		fn set_sram( &mut self ) {
			set_bit(&mut self.header[6], 1)
		}
		
		fn set_trainer( &mut self ) {
			set_bit(&mut self.header[6], 2);
			self.trainer = generate_bytes( 512 );
		}
		
		fn set_fourscreen( &mut self ) {
			set_bit(&mut self.header[6], 3)
		}
		
		fn set_pc10( &mut self ) {
			set_bit(&mut self.header[7], 0)
		}
		
		fn set_vs( &mut self ) {
			set_bit(&mut self.header[7], 1)
		}
		
		fn set_mapper( &mut self, mapper: u8 ) {
			self.header[6] = ( self.header[6] & 0x0F ) | ( ( mapper & 0x0Fu8 ) << 4 );
			self.header[7] = ( self.header[7] & 0x0F ) | ( ( mapper & 0xF0u8 ) << 0 );
		}
		
		fn set_prg_ram_pages( &mut self, pages: u8 ) {
			self.header[8] = pages;
		}
		
		fn build(&self) -> Vec<u8> {
			let mut buf = self.header.clone();
			buf.extend(self.trainer.iter().clone());
			buf.extend(self.prg_rom.iter().clone());
			buf.extend(self.chr_rom.iter().clone());
			buf
		}
	}
	
	#[test]
	#[should_panic]
	fn parse_panics_on_empty_input() {
		Rom::parse( &vec!() );
	}
	
	#[test]
	fn test_prg_rom() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( &builder.build() ).prg_rom, vec!() );
		
		builder.set_prg_page_count( 3 );
		assert_eq!( Rom::parse( &builder.build() ).prg_rom, builder.prg_rom );
	}
	
	#[test]
	fn test_chr_rom() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( &builder.build() ).chr_rom, vec!() );
		
		builder.set_chr_page_count( 150 );
		assert_eq!( Rom::parse( &builder.build() ).chr_rom, builder.chr_rom );
	}
	
	#[test]
	fn test_screen_mode_without_fourscreen() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( &builder.build() ).screen_mode(), ScreenMode::Horizontal );
		
		builder.set_mirroring();
		assert_eq!( Rom::parse( &builder.build() ).screen_mode(), ScreenMode::Vertical );
	}
	
	#[test]
	fn test_screen_mode_with_fourscreen() {
		let mut builder = RomBuilder::new();
		builder.set_fourscreen();
		assert_eq!( Rom::parse( &builder.build() ).screen_mode(), ScreenMode::FourScreen );
		
		builder.set_mirroring();
		assert_eq!( Rom::parse( &builder.build() ).screen_mode(), ScreenMode::FourScreen );
	}
	
	#[test]
	fn test_sram() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( &builder.build() ).sram(), false );
		
		builder.set_sram();
		assert_eq!( Rom::parse( &builder.build() ).sram(), true );
	}
	
	#[test]
	fn test_trainer() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( &builder.build() ).trainer(), &vec!() );
		
		builder.set_trainer();
		assert_eq!( Rom::parse( &builder.build() ).trainer().len(), builder.trainer.len() );
		assert_eq!( Rom::parse( &builder.build() ).trainer(), &builder.trainer );
	}
	
		
	#[test]
	fn test_pc10() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( &builder.build() ).pc10(), false );
		
		builder.set_pc10();
		assert_eq!( Rom::parse( &builder.build() ).pc10(), true );
	}
	
	#[test]
	fn test_vs() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( &builder.build() ).vs(), false );
		
		builder.set_vs();
		assert_eq!( Rom::parse( &builder.build() ).vs(), true );
	}
	
	#[test]
	fn test_mapper() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( &builder.build() ).mapper(), 0x00u8 );
		
		builder.set_mapper(0x0Au8);
		assert_eq!( Rom::parse( &builder.build() ).mapper(), 0x0Au8 );
		
		builder.set_mapper(0xF0u8);
		println!( "0x{:02X}, 0x{:02X}", builder.header[6], builder.header[7] );
		assert_eq!( Rom::parse( &builder.build() ).mapper(), 0xF0u8 );
	}
	
		#[test]
	fn test_prg_ram_pages() {
		let mut builder = RomBuilder::new();
		builder.set_prg_ram_pages(1);
		assert_eq!( Rom::parse( &builder.build() ).prg_ram.len(), PRG_RAM_PAGE_SIZE );
		
		builder.set_prg_ram_pages(0);
		assert_eq!( Rom::parse( &builder.build() ).prg_ram.len(), PRG_RAM_PAGE_SIZE );
		
		builder.set_prg_ram_pages(15);
		assert_eq!( Rom::parse( &builder.build() ).prg_ram.len(), 15 * PRG_RAM_PAGE_SIZE );
	}
}