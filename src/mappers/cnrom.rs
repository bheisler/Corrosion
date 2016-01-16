use super::*;

const CHR_BANK_SIZE: usize = 0x2000;

pub struct CNROM {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    chr_bank: u8,
}

impl CNROM {
    pub fn new(params: MapperParams) -> CNROM {
        CNROM {
            prg_rom: params.prg_rom,
            chr_rom: params.chr_rom,
            chr_bank: 0,
        }
    }
}

impl Mapper for CNROM {
    fn prg_read(&self, idx: u16) -> u8 {
        match idx {
            0x8000...0xFFFF => self.prg_rom[((idx - 0x8000) as usize % self.prg_rom.len())],
            x => invalid_address!(x),
        }
    }

    fn prg_write(&mut self, idx: u16, val: u8) {
        match idx {
            0x8000...0xFFFF => self.chr_bank = val,
            x => invalid_address!(x),
        }
    }

    fn chr_read(&self, idx: u16) -> u8 {
        let idx: usize = idx as usize;
        let bank = self.chr_bank as usize * CHR_BANK_SIZE;
        let idx = bank + idx;
        self.chr_rom[idx as usize % self.chr_rom.len()]
    }

    #[allow(unused_variables)]
    fn chr_write(&mut self, idx: u16, val: u8) {
        // Do Nothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mappers::{Mapper, MapperParams};

    #[test]
    fn test_can_create_mapper_0() {
        CNROM::new(MapperParams::simple(vec![], vec![]));
    }

    fn create_test_mapper() -> CNROM {
        CNROM::new(MapperParams::simple(vec!(0u8; 0x4000), vec!(0u8; 0x4000)))
    }

    #[test]
    fn test_prg_rom_read() {
        let prg_rom: Vec<_> = (0..0x4000)
                                  .map(|val| (val % 0xFF) as u8)
                                  .collect();
        let mapper = CNROM::new(MapperParams::simple(prg_rom, vec!(0u8; 0x4000)));

        assert_eq!(mapper.prg_read(0x8111), mapper.prg_read(0xC111));
    }

    #[test]
    fn test_prg_rom_mirroring() {
        let mut prg_rom: Vec<_> = vec!(0u8; 0x4000);
        prg_rom[0x2612] = 0x15;
        let mapper = CNROM::new(MapperParams::simple(prg_rom, vec!(0u8; 0x1000)));
        assert_eq!(mapper.prg_read(0xA612), 0x15);
    }

    #[test]
    fn test_prg_rom_write() {
        let mut mapper = create_test_mapper();

        mapper.prg_write(0x8612, 15);
        assert_eq!(mapper.prg_read(0x8612), 0);
    }

    #[test]
    fn test_chr_rom_read() {
        let chr_rom: Vec<_> = (0..0x2000)
                                  .map(|val| (val % 0xFF) as u8)
                                  .collect();
        let mapper = CNROM::new(MapperParams::simple(vec!(0u8; 0x4000), chr_rom));

        assert_eq!(mapper.prg_read(0x8111), mapper.prg_read(0xC111));
    }

    #[test]
    fn test_chr_rom_bankswitching() {
        fn adjust_val(val: usize) -> u8 {
            let val = val % 0xFF;
            let val = if val > 2000 {
                val - 1
            } else {
                val + 1
            };
            val as u8
        }

        let chr_rom: Vec<_> = (0..0x4000)
                                  .map(|val| adjust_val(val))
                                  .collect();
        let params = MapperParams::simple(vec!(0u8; 0x4000), chr_rom.clone());
        let mut mapper = CNROM::new(params);

        mapper.prg_write(0x8000, 0x00);
        assert_eq!(mapper.chr_read(0x0010), chr_rom[0x0010]);
        mapper.prg_write(0x8000, 0x01);
        assert_eq!(mapper.chr_read(0x0010), chr_rom[0x2010]);
    }

    #[test]
    fn test_chr_rom_write() {
        let mut mapper = create_test_mapper();

        mapper.chr_write(0x1612, 15);
        assert_eq!(mapper.chr_read(0x1612), 0);
    }
}
