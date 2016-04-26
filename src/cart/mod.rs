pub mod ines;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::io;

use cart::ines::{Rom, RomError};
use mappers::{Mapper, MapperParams};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ScreenMode {
    Horizontal,
    Vertical,
    FourScreen,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum System {
    NES,
    Vs,
    PC10,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TvFormat {
    NTSC,
    PAL,
}

pub struct Cart {
    mapper: Box<Mapper>,
    pub mode: ScreenMode,
    pub system: System,
    pub tv: TvFormat,
}

quick_error! {
    #[derive(Debug)]
    pub enum RomReadError {
        Io(err: io::Error) {
            display("IO Error: {}", err)
            description(err.description())
            cause(err)
            from()
        }
        Parse(err: RomError) {
            display("ROM Error: {}", err)
            description(err.description())
            cause(err)
            from()
        }
    }
}

impl Cart {
    pub fn prg_read(&mut self, idx: u16) -> u8 {
        self.mapper.prg_read(idx)
    }
    pub fn prg_write(&mut self, idx: u16, val: u8) {
        self.mapper.prg_write(idx, val)
    }
    pub fn chr_read(&mut self, idx: u16) -> u8 {
        self.mapper.chr_read(idx)
    }
    pub fn chr_write(&mut self, idx: u16, val: u8) {
        self.mapper.chr_write(idx, val)
    }
    pub fn vram_mask(&self) -> u16 {
        match self.mode {
            ScreenMode::Horizontal => 0xFBFF,
            ScreenMode::Vertical => 0xF7FF,
            _ => panic!("unsupported mirroring mode."),
        }
    }

    pub fn new(mapper: Box<Mapper>) -> Cart {
        Cart {
            mapper: mapper,
            mode: ScreenMode::Horizontal,
            system: System::NES,
            tv: TvFormat::NTSC,
        }
    }

    pub fn read(path: &Path) -> Result<Cart, RomReadError> {
        let mut file = try!(File::open(path));
        let mut buf = vec![];
        try!(file.read_to_end(&mut buf));
        let rom = try!(Rom::parse(&buf));

        let mapper = rom.mapper();
        let screen_mode = rom.screen_mode();
        let system = rom.system();
        let tv = rom.tv_system();
        let (prg_rom, chr_rom, prg_ram_size) = (rom.prg_rom, rom.chr_rom, rom.prg_ram_size);

        let params = MapperParams {
            prg_rom: prg_rom,
            chr_rom: chr_rom,

            prg_ram_size: prg_ram_size,

            rom_path: path,
        };

        let mapper = Mapper::new(mapper as u16, params);
        Ok(Cart {
            mapper: mapper,
            mode: screen_mode,
            system: system,
            tv: tv,
        })
    }
}
