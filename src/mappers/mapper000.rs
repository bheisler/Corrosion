use super::*;

struct Mapper000 {
    prg_rom: Box<[u8]>,
    chr_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
}

pub fn new(params: MapperParams) -> Box<Mapper> {
    Box::new(Mapper000 {
        prg_rom: params.prg_rom.into_boxed_slice(),
        chr_rom: params.chr_rom.into_boxed_slice(),
        prg_ram: vec![0u8; params.prg_ram_size].into_boxed_slice(),
    })
}

impl Mapper for Mapper000 {
    fn prg_read(&self, idx: u16) -> u8 {
        match idx {
            0x6000...0x7FFF => self.prg_ram[((idx - 0x6000) as usize % self.prg_ram.len())],
            0x8000...0xFFFF => self.prg_rom[((idx - 0x8000) as usize % self.prg_rom.len())],
            x => invalid_address!(x),
        }
    }

    fn prg_write(&mut self, idx: u16, val: u8) {
        match idx {
            0x6000...0x7FFF => {
                let idx = (idx - 0x6000) as usize % self.prg_ram.len();
                self.prg_ram[idx] = val;
            }
            0x8000...0xFFFF => (),//Do nothing
            x => invalid_address!(x),
        }
    }

    fn chr_read(&self, idx: u16) -> u8 {
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
        new(MapperParams::simple(vec![], vec![]));
    }

    fn create_test_mapper() -> Box<Mapper> {
        new(MapperParams::simple(vec!(0u8; 0x4000), vec!(0u8; 0x4000)))
    }

    #[test]
    fn test_prg_ram_read_write() {
        let mut params = MapperParams::simple(vec!(0u8; 0x4000), vec!(0u8; 0x4000));
        params.prg_ram_size = 0x1000;
        let mut nrom = new(params);
        nrom.prg_write(0x6111, 15);
        assert_eq!(nrom.prg_read(0x6111), 15);

        nrom.prg_write(0x6112, 16);
        assert_eq!(nrom.prg_read(0x7112), 16);
    }

    #[test]
    fn test_prg_rom_read() {
        let prg_rom: Vec<_> = (0..0x4000)
                                  .map(|val| (val % 0xFF) as u8)
                                  .collect();
        let mapper = new(MapperParams::simple(prg_rom, vec!(0u8; 0x4000)));

        assert_eq!(mapper.prg_read(0x8111), mapper.prg_read(0xC111));
    }

    #[test]
    fn test_prg_rom_mirroring() {
        let mut prg_rom: Vec<_> = vec!(0u8; 0x4000);
        prg_rom[0x2612] = 0x15;
        let mapper = new(MapperParams::simple(prg_rom, vec!(0u8; 0x1000)));
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
        let mapper = new(MapperParams::simple(vec!(0u8; 0x4000), chr_rom));

        assert_eq!(mapper.prg_read(0x8111), mapper.prg_read(0xC111));
    }

    #[test]
    fn test_chr_rom_write() {
        let mut mapper = create_test_mapper();

        mapper.chr_write(0x1612, 15);
        assert_eq!(mapper.chr_read(0x1612), 0);
    }
}
