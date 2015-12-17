static MAGIC_NUMBERS : [u8; 4] = [0x4Eu8, 0x45u8, 0x53u8, 0x1Au8];

pub struct Rom {
	prg_rom_pages: u8,
	chr_rom_pages: u8
}

impl Rom {
	pub fn parse(data: Vec<u8>) -> Rom {
		assert_eq!( &data[0 .. 4], MAGIC_NUMBERS );
		let prg_pages = data[4];
		let chr_pages = data[5];
		Rom { 
			prg_rom_pages: prg_pages,
			chr_rom_pages: chr_pages 
		}
	} 
}

#[cfg(test)]
mod tests {
	use super::*;
	
	fn create_binary_rom( prg_page_count: u8, chr_page_count: u8 ) -> Vec<u8> {
		let mut bin_rom = vec![0x4Eu8, 0x45u8, 0x53u8, 0x1Au8];
		bin_rom.push( prg_page_count );
		bin_rom.push( chr_page_count );
		bin_rom
	}
	
	#[test]
	#[should_panic]
	fn parse_panics_on_empty_input() {
		Rom::parse( vec!() );
	}
	
	#[test]
	fn parse_succeeds_on_input_with_magic_numbers() {
		Rom::parse( create_binary_rom( 0, 0 ) );
	}
	
	#[test]
	fn parse_reads_prg_rom_page_count() {
		let rom = Rom::parse( create_binary_rom( 12, 0 ) );
		assert_eq!( rom.prg_rom_pages, 12 );
	}
	
	#[test]
	fn parse_reads_chr_rom_page_count() {
		let rom = Rom::parse( create_binary_rom( 12, 15 ) );
		assert_eq!( rom.chr_rom_pages, 15 );
	}
}