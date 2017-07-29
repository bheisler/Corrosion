mod volatile;
mod battery;

mod bank;

mod mapper000;
mod mmc1;

use cart::ScreenMode;
use cpu::dispatcher::Dispatcher;
pub use mappers::bank::RomBank;
use std::cell::UnsafeCell;
use std::path::Path;
use std::rc::Rc;

static VERTICAL: [u16; 4] = [0x2000, 0x2400, 0x2000, 0x2400];
static HORIZONTAL: [u16; 4] = [0x2000, 0x2000, 0x2400, 0x2400];
static ONE_SCREEN_LOW: [u16; 4] = [0x2000, 0x2000, 0x2000, 0x2000];
static ONE_SCREEN_HIGH: [u16; 4] = [0x2400, 0x2400, 0x2400, 0x2400];
static FOUR_SCREEN: [u16; 4] = [0x2000, 0x2400, 0x2800, 0x2C00];

fn standard_mapping_tables(mode: ScreenMode) -> &'static [u16; 4] {
    match mode {
        ScreenMode::Vertical => &VERTICAL,
        ScreenMode::Horizontal => &HORIZONTAL,
        ScreenMode::OneScreenHigh => &ONE_SCREEN_HIGH,
        ScreenMode::OneScreenLow => &ONE_SCREEN_LOW,
        ScreenMode::FourScreen => &FOUR_SCREEN,
    }
}

pub trait Mapper {
    fn prg_rom_read(&mut self, idx: u16) -> &RomBank;
    fn prg_rom_write(&mut self, idx: u16, val: u8) -> &mut RomBank;

    fn prg_ram_read(&mut self, idx: u16) -> u8;
    fn prg_ram_write(&mut self, idx: u16, val: u8);

    fn chr_read(&mut self, idx: u16) -> u8;
    fn chr_write(&mut self, idx: u16, val: u8);

    fn get_mirroring_table(&self) -> &[u16; 4];

    fn set_dispatcher(&mut self, dispatcher: Rc<UnsafeCell<Dispatcher>>);
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

#[cfg(test)]
pub fn create_test_mapper(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mode: ScreenMode) -> Box<Mapper> {
    let path_buf = ::std::path::PathBuf::new();
    let path = path_buf.as_path();
    let mut params = MapperParams::simple(path, prg_rom, chr_rom);
    params.mirroring_mode = mode;
    Mapper::new(0, params)
}
