static MAGIC_NUMBERS : [u8; 4] = [0x4Eu8, 0x45u8, 0x53u8, 0x1Au8];

#[derive(PartialEq, Debug)]
pub enum ScreenMode {
	Horizontal,
	Vertical,
	FourScreen
}

pub struct Rom {
	prg_pages: u8,
	chr_pages: u8,
	flags6: u8,
	flags7: u8,
}

impl Rom {
	pub fn parse(data: Vec<u8>) -> Rom {
		assert_eq!( &data[0 .. 4], MAGIC_NUMBERS );
		let prg_pages = data[4];
		let chr_pages = data[5];
		let flags6 = data[6];
		let flags7 = data[7];
		Rom { 
			prg_pages: prg_pages,
			chr_pages: chr_pages,
			flags6: flags6,
			flags7: flags7
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
	
	fn get_bit(&self, byte: u8, bit_num: u8) -> bool {
		!((byte & 1u8 << bit_num) == 0)
	}
	
	pub fn sram(&self) -> bool {
		self.get_bit(self.flags6, 1)
	}
	
	pub fn trainer(&self) -> bool {
		self.get_bit(self.flags6, 2)
	}
	
	pub fn pc10(&self) -> bool {
		self.get_bit(self.flags7, 0)
	}
	
	pub fn vs(&self) -> bool {
		self.get_bit(self.flags7, 1)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	struct RomBuilder {
		header : Vec<u8>
	}
	
	fn set_bit(byte: &mut u8, bit_num: u8) {
		*byte = *byte | 1u8 << bit_num;
	}
	
	impl RomBuilder {
		fn new() -> RomBuilder {
			let mut header = vec![0x4Eu8, 0x45u8, 0x53u8, 0x1Au8];
			header.extend([0; 12].iter().cloned());
			return RomBuilder{ header: header }
		}
		
		fn set_prg_page_count( &mut self, count: u8 ) {
			self.header[4] = count;
		}
		
		fn set_chr_page_count( &mut self, count: u8 ) {
			self.header[5] = count;
		}
		
		fn set_mirroring( &mut self ) {
			set_bit(&mut self.header[6], 0)
		}
		
		fn set_sram( &mut self ) {
			set_bit(&mut self.header[6], 1)
		}
		
		fn set_trainer( &mut self ) {
			set_bit(&mut self.header[6], 2)
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
		
		fn build(&self) -> Vec<u8> {
			self.header.to_vec()
		}
	}
	
	#[test]
	#[should_panic]
	fn parse_panics_on_empty_input() {
		Rom::parse( vec!() );
	}
	
	#[test]
	fn parse_reads_prg_pages() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( builder.build() ).prg_pages, 0 );
		
		builder.set_prg_page_count( 150 );
		assert_eq!( Rom::parse( builder.build() ).prg_pages, 150 );
	}
	
	#[test]
	fn parse_reads_chr_pages() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( builder.build() ).chr_pages, 0 );
		
		builder.set_chr_page_count( 150 );
		assert_eq!( Rom::parse( builder.build() ).chr_pages, 150 );
	}
	
	#[test]
	fn test_screen_mode_without_fourscreen() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( builder.build() ).screen_mode(), ScreenMode::Horizontal );
		
		builder.set_mirroring();
		assert_eq!( Rom::parse( builder.build() ).screen_mode(), ScreenMode::Vertical );
	}
	
	#[test]
	fn test_screen_mode_with_fourscreen() {
		let mut builder = RomBuilder::new();
		builder.set_fourscreen();
		assert_eq!( Rom::parse( builder.build() ).screen_mode(), ScreenMode::FourScreen );
		
		builder.set_mirroring();
		assert_eq!( Rom::parse( builder.build() ).screen_mode(), ScreenMode::FourScreen );
	}
	
	#[test]
	fn test_sram() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( builder.build() ).sram(), false );
		
		builder.set_sram();
		assert_eq!( Rom::parse( builder.build() ).sram(), true );
	}
	
	#[test]
	fn test_trainer() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( builder.build() ).trainer(), false );
		
		builder.set_trainer();
		assert_eq!( Rom::parse( builder.build() ).trainer(), true );
	}
	
		
	#[test]
	fn test_pc10() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( builder.build() ).pc10(), false );
		
		builder.set_pc10();
		assert_eq!( Rom::parse( builder.build() ).pc10(), true );
	}
	
	#[test]
	fn test_vs() {
		let mut builder = RomBuilder::new();
		assert_eq!( Rom::parse( builder.build() ).vs(), false );
		
		builder.set_vs();
		assert_eq!( Rom::parse( builder.build() ).vs(), true );
	}
}