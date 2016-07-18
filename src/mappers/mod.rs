mod volatile;
mod battery;

mod mapper000;
mod mmc1;

use super::memory::MemSegment;
use std::path::Path;
use cart::ScreenMode;

pub trait Mapper {
    fn prg_read(&mut self, idx: u16) -> u8;
    fn prg_write(&mut self, idx: u16, val: u8);
    fn chr_read(&mut self, idx: u16) -> u8;
    fn chr_write(&mut self, idx: u16, val: u8);

    fn get_mirroring_mode(&self) -> ScreenMode;
}

pub struct MapperParams<'a> {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,

    pub prg_ram_size: usize,

    pub rom_path: &'a Path,

    pub has_battery_backed_ram: bool,
    pub mirroring_mode: ScreenMode,
}

impl<'a> MapperParams<'a> {
    #[cfg(test)]
    pub fn simple(rom_path: &'a Path, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> MapperParams<'a> {
        MapperParams {
            prg_rom: prg_rom,
            chr_rom: chr_rom,

            prg_ram_size: 0x2000,

            rom_path: rom_path,

            has_battery_backed_ram: false,
            mirroring_mode: ScreenMode::OneScreenLow,
        }
    }
}

impl Mapper {
    pub fn new(id: u16, params: MapperParams) -> Box<Mapper> {
        match id {
            0 => mapper000::new(params),
            1 => mmc1::new(params),
            m => panic!("Unsupported Mapper: {}", m),
        }
    }
}

impl MemSegment for Mapper {
    fn read(&mut self, idx: u16) -> u8 {
        self.prg_read(idx)
    }
    fn write(&mut self, idx: u16, val: u8) {
        self.prg_write(idx, val)
    }
}

#[cfg(test)]
pub fn create_test_mapper(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mode: ScreenMode) -> Box<Mapper> {
    let path_buf = ::std::path::PathBuf::new();
    let path = path_buf.as_path();
    let mut params = MapperParams::simple(path, prg_rom, chr_rom);
    params.mirroring_mode = mode;
    Mapper::new(0, params)
}
