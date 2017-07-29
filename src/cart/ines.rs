use cart::*;
use nom::{IResult, be_u8, ErrorKind};
use std::convert::Into;

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

bitflags! {
    struct Flags6: u8 {
        const VERTICAL =    0b0000_0001;
        const SRAM =        0b0000_0010;
        const TRAINER =     0b0000_0100;
        const FOUR_SCREEN = 0b0000_1000;
    }
}

impl Into<ScreenMode> for Flags6 {
    fn into(self) -> ScreenMode {
        if self.contains(FOUR_SCREEN) {
            ScreenMode::FourScreen
        } else if self.contains(VERTICAL) {
            ScreenMode::Vertical
        } else {
            ScreenMode::Horizontal
        }
    }
}

bitflags! {
    struct Flags7: u8 {
        const VS_UNISYSTEM =  0b0000_0001;
        const PLAYCHOICE_10 = 0b0000_0010;
        const NES_2 =         0b0000_1100;
    }
}

impl Into<System> for Flags7 {
    fn into(self) -> System {
        if self.contains(VS_UNISYSTEM) {
            System::Vs
        } else if self.contains(PLAYCHOICE_10) {
            System::PC10
        } else {
            System::NES
        }
    }
}

bitflags! {
    struct Flags9: u8 {
        const PAL = 0b0000_0001;
    }
}

impl Into<TvFormat> for Flags9 {
    fn into(self) -> TvFormat {
        if self.contains(PAL) {
            TvFormat::PAL
        } else {
            TvFormat::NTSC
        }
    }
}

pub struct Rom {
    mapper: u8,
    screen_mode: ScreenMode,
    sram: bool,
    system: System,
    tv_format: TvFormat,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub prg_ram_size: usize,
}

fn validate_not_nes2(input: &[u8], flags_7: Flags7) -> IResult<&[u8], ()> {
    if (flags_7.bits() & 0b0000_1100) == 0b0000_1000 {
        IResult::Error(error_code!(ErrorKind::Custom(1)))
    } else {
        IResult::Done(input, ())
    }
}

fn parse_rom(input: &[u8]) -> IResult<&[u8], Rom> {
    do_parse!(input,
        tag!(b"NES\x1A") >>
        prg_pages: be_u8 >>
        chr_pages: be_u8 >>
        flags_6: bits!(tuple!(
            take_bits!(u8, 4),
            map_opt!(take_bits!(u8, 4), Flags6::from_bits))) >>
        flags_7: bits!(tuple!(
            take_bits!(u8, 4),
            map_opt!(take_bits!(u8, 4), Flags7::from_bits))) >>
        call!(validate_not_nes2, flags_7.1) >>
        prg_ram_pages: be_u8 >>
        flags_9: map_opt!(be_u8, Flags9::from_bits) >>
        tag!([0u8; 6]) >>
        //Skip the trainer if there is one
        cond!(flags_6.1.contains(TRAINER), take!(TRAINER_LENGTH)) >>
        prg_rom: take!(prg_pages as usize * PRG_ROM_PAGE_SIZE) >>
        chr_rom: take!(chr_pages as usize * CHR_ROM_PAGE_SIZE) >>
        ( Rom {
            mapper: (flags_7.0 << 4) | flags_6.0,
            screen_mode: flags_6.1.into(),
            sram: flags_6.1.contains(SRAM),
            system: flags_7.1.into(),
            tv_format: flags_9.into(),
            prg_rom: prg_rom.into(),
            chr_rom: chr_rom.into(),
            prg_ram_size: if prg_ram_pages == 0 {
                PRG_RAM_PAGE_SIZE
            } else {
                prg_ram_pages as usize * PRG_RAM_PAGE_SIZE
            },
        } )
    )
}

impl Rom {
    /// Parse the given bytes as an iNES 1.0 header.
    /// NES 2.0 is not supported yet.
    pub fn parse(data: &[u8]) -> Result<Rom, RomError> {
        match parse_rom(data) {
            IResult::Done(_, rom) => Ok(rom),
            IResult::Error(err) => {
                match err {
                    ErrorKind::Custom(_) => Err(RomError::Nes2NotSupported),
                    _ => Err(RomError::DamagedHeader),
                }
            }
            IResult::Incomplete(_) => Err(RomError::UnexpectedEndOfData),
        }
    }

    pub fn screen_mode(&self) -> ScreenMode {
        self.screen_mode
    }

    pub fn sram(&self) -> bool {
        self.sram
    }

    pub fn system(&self) -> System {
        self.system
    }

    pub fn tv_system(&self) -> TvFormat {
        self.tv_format
    }

    pub fn mapper(&self) -> u8 {
        self.mapper
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    use self::rand::{Rng, thread_rng};
    use super::*;

    struct RomBuilder {
        header: Vec<u8>,
        trainer: Vec<u8>,
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
    }

    fn set_bit(byte: &mut u8, bit_num: u8) {
        *byte |= 1u8 << bit_num;
    }

    fn generate_bytes(size: usize) -> Vec<u8> {
        let mut rng = thread_rng();
        let mut bytes: Vec<u8> = vec![0u8; size];
        rng.fill_bytes(&mut bytes);
        bytes
    }

    impl RomBuilder {
        fn new() -> RomBuilder {
            let mut header = b"NES\x1A".to_vec();
            header.extend([0; 12].iter().cloned());

            RomBuilder {
                header: header,
                trainer: vec![],
                prg_rom: vec![],
                chr_rom: vec![],
            }
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
            self.header[7] = (self.header[7] & 0x0F) | ((mapper & 0xF0u8));
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
        assert!(Rom::parse(&Vec::new()).err().unwrap() == RomError::UnexpectedEndOfData);
    }

    #[test]
    fn parse_returns_failure_on_incorrect_magic_bytes() {
        let mut builder = RomBuilder::new();
        builder.set_prg_page_count(3);
        let mut buf = builder.build();
        buf[3] = b'0';
        assert!(Rom::parse(&buf).err().unwrap() == RomError::DamagedHeader);
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

        builder.set_prg_page_count(4);
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
