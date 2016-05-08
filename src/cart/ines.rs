use cart::*;

const MAGIC_NUMBERS: [u8; 4] = [0x4Eu8, 0x45u8, 0x53u8, 0x1Au8];
pub const PRG_ROM_PAGE_SIZE: usize = 16384;
pub const CHR_ROM_PAGE_SIZE: usize = 8192;
pub const PRG_RAM_PAGE_SIZE: usize = 8192;
pub const TRAINER_LENGTH: usize = 512;

quick_error! {
    #[derive(Debug, PartialEq)]
	pub enum RomError {
        DamagedHeader {
            description("ROM data had missing or damaged header.")
        }
        UnexpectedEndOfData {
            description("Unexpected end of data.")
        }
        Nes2NotSupported {
            description("NES 2.0 ROMs are not currently supported.")
        }
    }
}

pub struct Rom {
    flags6: u8,
    flags7: u8,
    flags9: u8,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub prg_ram_size: usize,
}

fn get_bit(byte: u8, bit_num: u8) -> bool {
    !((byte & 1u8 << bit_num) == 0)
}

fn read_byte(iter: &mut Iterator<Item = u8>) -> Result<u8, RomError> {
    match iter.next() {
        Some(val) => Ok(val),
        None => Err(RomError::UnexpectedEndOfData),
    }
}

fn read_bytes(iter: &mut Iterator<Item = u8>, bytes: usize) -> Result<Vec<u8>, RomError> {
    let buf: Vec<_> = iter.take(bytes).collect();
    if buf.len() < bytes {
        Err(RomError::UnexpectedEndOfData)
    } else {
        Ok(buf)
    }
}

impl Rom {
    ///Parse the given bytes as an iNES 1.0 header.
    ///NES 2.0 is not supported yet.
    pub fn parse(data: &[u8]) -> Result<Rom, RomError> {
        let mut iter = data.iter().cloned();
        if try!(read_bytes(&mut iter, 4)) != MAGIC_NUMBERS {
            return Err(RomError::DamagedHeader);
        }
        let prg_rom_pages = try!(read_byte(&mut iter));
        let chr_rom_pages = try!(read_byte(&mut iter));
        let flags6 = try!(read_byte(&mut iter));
        let flags7 = try!(read_byte(&mut iter));

        if (flags7 & 0b0000_1100u8) == 0b0000_1000u8 {
            return Err(RomError::Nes2NotSupported);
        }

        let prg_ram_pages = match try!(read_byte(&mut iter)) {
            0 => 1,
            x => x,
        };

        let flags9 = try!(read_byte(&mut iter));

        if try!(read_bytes(&mut iter, 6)) != vec![0u8; 6] {
            return Err(RomError::DamagedHeader);
        }

        if get_bit(flags6, 2) {
            // Discard trainer for now, can add support later if needed
            let _ = try!(read_bytes(&mut iter, TRAINER_LENGTH));
        };

        Ok(Rom {
            prg_rom: try!(read_bytes(&mut iter, PRG_ROM_PAGE_SIZE * prg_rom_pages as usize)),
            chr_rom: try!(read_bytes(&mut iter, CHR_ROM_PAGE_SIZE * chr_rom_pages as usize)),
            flags6: flags6,
            flags7: flags7,
            flags9: flags9,
            prg_ram_size: PRG_RAM_PAGE_SIZE * prg_ram_pages as usize,
        })
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

    pub fn system(&self) -> System {
        if get_bit(self.flags7, 0) {
            System::Vs
        } else if get_bit(self.flags7, 1) {
            System::PC10
        } else {
            System::NES
        }
    }

    pub fn tv_system(&self) -> TvFormat {
        if get_bit(self.flags9, 0) {
            TvFormat::PAL
        } else {
            TvFormat::NTSC
        }
    }

    pub fn mapper(&self) -> u8 {
        ((self.flags6 & 0xF0) >> 4) | (self.flags7 & 0xF0)
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    use super::*;
    use cart::*;
    use self::rand::{Rng, thread_rng};

    struct RomBuilder {
        header: Vec<u8>,
        trainer: Vec<u8>,
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
    }

    fn set_bit(byte: &mut u8, bit_num: u8) {
        *byte = *byte | 1u8 << bit_num;
    }

    fn generate_bytes(size: usize) -> Vec<u8> {
        let mut rng = thread_rng();
        let mut bytes: Vec<u8> = vec!(0u8; size);
        rng.fill_bytes(&mut bytes);
        bytes
    }

    impl RomBuilder {
        fn new() -> RomBuilder {
            let mut header = vec![0x4Eu8, 0x45u8, 0x53u8, 0x1Au8];
            header.extend([0; 12].iter().cloned());
            return RomBuilder {
                header: header,
                trainer: vec![],
                prg_rom: vec![],
                chr_rom: vec![],
            };
        }

        fn set_prg_page_count(&mut self, count: u8) {
            self.header[4] = count;
            self.prg_rom = generate_bytes(count as usize * PRG_ROM_PAGE_SIZE);
        }

        fn set_chr_page_count(&mut self, count: u8) {
            self.header[5] = count;
            self.chr_rom = generate_bytes(count as usize * CHR_ROM_PAGE_SIZE);
        }

        fn set_mirroring(&mut self) {
            set_bit(&mut self.header[6], 0)
        }

        fn set_sram(&mut self) {
            set_bit(&mut self.header[6], 1)
        }

        fn set_fourscreen(&mut self) {
            set_bit(&mut self.header[6], 3)
        }

        fn set_pc10(&mut self) {
            set_bit(&mut self.header[7], 1)
        }

        fn set_vs(&mut self) {
            set_bit(&mut self.header[7], 0)
        }

        fn set_mapper(&mut self, mapper: u8) {
            self.header[6] = (self.header[6] & 0x0F) | ((mapper & 0x0Fu8) << 4);
            self.header[7] = (self.header[7] & 0x0F) | ((mapper & 0xF0u8) << 0);
        }

        fn set_prg_ram_pages(&mut self, pages: u8) {
            self.header[8] = pages;
        }

        fn set_nes2(&mut self) {
            self.header[7] = (self.header[7] & 0b00001100) | 0b0000_1000;
        }

        fn set_pal(&mut self) {
            set_bit(&mut self.header[9], 0)
        }

        fn build(&self) -> Vec<u8> {
            let mut buf = self.header.clone();
            buf.extend(self.trainer.iter().clone());
            buf.extend(self.prg_rom.iter().clone());
            buf.extend(self.chr_rom.iter().clone());
            buf
        }

        fn build_rom(&self) -> Rom {
            Rom::parse(&self.build()).unwrap()
        }
    }

    #[test]
    fn parse_returns_failure_on_empty_input() {
        assert!(Rom::parse(&vec![]).err().unwrap() == RomError::UnexpectedEndOfData);
    }

    #[test]
    fn parse_returns_failure_on_partial_input() {
        let mut builder = RomBuilder::new();
        builder.set_prg_page_count(3);
        let mut buf = builder.build();
        buf.truncate(300);
        assert!(Rom::parse(&buf).err().unwrap() == RomError::UnexpectedEndOfData);
    }

    #[test]
    fn parse_returns_failure_on_damaged_input() {
        let mut buf = RomBuilder::new().build();
        buf[2] = 155;
        assert!(Rom::parse(&buf).err().unwrap() == RomError::DamagedHeader);
    }

    #[test]
    fn parse_returns_failure_on_nes2_input() {
        let mut builder = RomBuilder::new();
        builder.set_nes2();
        let buf = builder.build();
        assert!(Rom::parse(&buf).err().unwrap() == RomError::Nes2NotSupported);
    }

    #[test]
    fn test_prg_rom() {
        let mut builder = RomBuilder::new();
        assert_eq!(&builder.build_rom().prg_rom, &vec![]);

        builder.set_prg_page_count(3);
        assert_eq!(&builder.build_rom().prg_rom, &builder.prg_rom);
    }

    #[test]
    fn test_chr_rom() {
        let mut builder = RomBuilder::new();
        assert_eq!(&builder.build_rom().chr_rom, &vec![]);

        builder.set_chr_page_count(150);
        assert_eq!(&builder.build_rom().chr_rom, &builder.chr_rom);
    }

    #[test]
    fn test_screen_mode_without_fourscreen() {
        let mut builder = RomBuilder::new();
        assert_eq!(builder.build_rom().screen_mode(), ScreenMode::Horizontal);

        builder.set_mirroring();
        assert_eq!(builder.build_rom().screen_mode(), ScreenMode::Vertical);
    }

    #[test]
    fn test_screen_mode_with_fourscreen() {
        let mut builder = RomBuilder::new();
        builder.set_fourscreen();
        assert_eq!(builder.build_rom().screen_mode(), ScreenMode::FourScreen);

        builder.set_mirroring();
        assert_eq!(builder.build_rom().screen_mode(), ScreenMode::FourScreen);
    }

    #[test]
    fn test_sram() {
        let mut builder = RomBuilder::new();
        assert_eq!(builder.build_rom().sram(), false);

        builder.set_sram();
        assert_eq!(builder.build_rom().sram(), true);
    }

    #[test]
    fn test_system() {
        let builder = RomBuilder::new();
        assert_eq!(builder.build_rom().system(), System::NES);

        let mut builder = RomBuilder::new();
        builder.set_vs();
        assert_eq!(builder.build_rom().system(), System::Vs);

        let mut builder = RomBuilder::new();
        builder.set_pc10();
        assert_eq!(builder.build_rom().system(), System::PC10);
    }

    #[test]
    fn test_tv_system() {
        let mut builder = RomBuilder::new();
        assert_eq!(builder.build_rom().tv_system(), TvFormat::NTSC);

        builder.set_pal();
        assert_eq!(builder.build_rom().tv_system(), TvFormat::PAL);
    }

    #[test]
    fn test_mapper() {
        let mut builder = RomBuilder::new();
        assert_eq!(builder.build_rom().mapper(), 0x00u8);

        builder.set_mapper(0x0Au8);
        assert_eq!(builder.build_rom().mapper(), 0x0Au8);

        builder.set_mapper(0xF0u8);
        assert_eq!(builder.build_rom().mapper(), 0xF0u8);
    }

    #[test]
    fn test_prg_ram_pages() {
        let mut builder = RomBuilder::new();
        builder.set_prg_ram_pages(1);
        assert_eq!(builder.build_rom().prg_ram_size, PRG_RAM_PAGE_SIZE);

        builder.set_prg_ram_pages(0);
        assert_eq!(builder.build_rom().prg_ram_size, PRG_RAM_PAGE_SIZE);

        builder.set_prg_ram_pages(15);
        assert_eq!(builder.build_rom().prg_ram_size, 15 * PRG_RAM_PAGE_SIZE);
    }
}
