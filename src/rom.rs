static MAGIC_NUMBERS : [u8; 4] = [0x4Eu8, 0x45u8, 0x53u8, 0x1Au8];

pub struct Rom {
	data: Vec<u8>,
}

impl Rom {
	pub fn parse(data: Vec<u8>) -> Rom {
		assert_eq!( &data[0 .. 4], MAGIC_NUMBERS );
		Rom{ data: data }
	} 
}

#[cfg(test)]
mod tests {
	
	use super::*;
	
	#[test]
	#[should_panic]
	fn parse_panics_on_empty_input() {
		Rom::parse( vec!() );
	}
	
	#[test]
	fn parse_succeeds_on_input_with_magic_numbers() {
		Rom::parse( vec![0x4Eu8, 0x45u8, 0x53u8, 0x1Au8] );
	}
}