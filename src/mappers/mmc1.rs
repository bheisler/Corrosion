use super::{Mapper, MapperParams};
use super::bank::*;
use memory::MemSegment;
use super::volatile::VolatileRam;
use super::battery::BatteryBackedRam;
use cart::ScreenMode;

#[derive(Debug, Clone, PartialEq)]
struct Ctrl {
    mode: PrgMode,
    mirroring: &'static [u16; 4], // TODO: Add chr mode
}

#[derive(Debug, Clone, PartialEq)]
enum PrgMode {
    Switch32Kb,
    FixFirst,
    FixLast,
}

#[derive(Debug, Clone, PartialEq)]
struct Regs {
    control: Ctrl,

    chr_0: u8,
    chr_1: u8,
    prg_bank: usize,
}

struct MMC1 {
    regs: Regs,

    accumulator: u8,
    write_counter: u8,

    prg_rom: MappingTable,
    chr_ram: Box<[u8]>,
    prg_ram: Box<MemSegment>,
}

impl MMC1 {
    fn update_mapping(&mut self) {
        match self.regs.control.mode {
            PrgMode::Switch32Kb => {
                self.prg_rom.map_pages_linear(0..8, (self.regs.prg_bank & 0b0000_1110) * 8)
            }
            PrgMode::FixFirst => {
                self.prg_rom.map_pages_linear(0..4, 0);
                self.prg_rom.map_pages_linear(4..8, (self.regs.prg_bank & 0b0000_1111) * 4);
            }
            PrgMode::FixLast => {
                self.prg_rom.map_pages_linear(0..4, (self.regs.prg_bank & 0b0000_1111) * 4);
                let bank_count = self.prg_rom.bank_count();
                self.prg_rom.map_pages_linear(4..8, bank_count - 4);
            }
        }
    }

    fn reset(&mut self) {
        self.accumulator = 0;
        self.write_counter = 0;
        self.regs.control = Ctrl {
            mode: PrgMode::FixLast,
            mirroring: super::standard_mapping_tables(ScreenMode::OneScreenLow),
        };
        self.update_mapping();
    }

    fn do_write(&mut self, idx: u16) {
        match idx {
            0x8000...0x9FFF => {
                let val = self.accumulator;
                let mode = match (val & 0x0C) >> 2 {
                    0 | 1 => PrgMode::Switch32Kb,
                    2 => PrgMode::FixFirst,
                    3 => PrgMode::FixLast,
                    _ => panic!("Can't happen."),
                };
                let mirroring = match val & 0x03 {
                    0 => ScreenMode::OneScreenLow,
                    1 => ScreenMode::OneScreenHigh,
                    2 => ScreenMode::Vertical,
                    3 => ScreenMode::Horizontal,
                    _ => panic!("Can't happen."),
                };
                self.regs.control = Ctrl {
                    mode: mode,
                    mirroring: super::standard_mapping_tables(mirroring),
                };
            }
            0xA000...0xBFFF => self.regs.chr_0 = self.accumulator,
            0xC000...0xDFFF => self.regs.chr_1 = self.accumulator,
            0xE000...0xFFFF => self.regs.prg_bank = self.accumulator as usize,
            x => invalid_address!(x),
        }
        self.update_mapping();
    }
}

fn prg_ram_addr(idx: u16) -> u16 {
    idx - 0x6000
}

pub fn new(params: MapperParams) -> Box<Mapper> {
    let chr_ram = if params.chr_rom.is_empty() {
        vec![0u8; 0x2000].into_boxed_slice()
    } else {
        vec![0u8; 0].into_boxed_slice()
    };

    let prg_ram: Box<MemSegment> = if params.has_battery_backed_ram {
        Box::new(BatteryBackedRam::new(params.rom_path, params.prg_ram_size as u32).unwrap())
    } else {
        Box::new(VolatileRam::new(params.prg_ram_size as usize))
    };

    let mut mapper = MMC1 {
        regs: Regs {
            control: Ctrl {
                mode: PrgMode::FixLast,
                mirroring: super::standard_mapping_tables(ScreenMode::OneScreenLow),
            },
            chr_0: 0,
            chr_1: 0,
            prg_bank: 0,
        },
        accumulator: 0,
        write_counter: 0,
        prg_rom: MappingTable::new(params.prg_rom),
        chr_ram: chr_ram,
        prg_ram: prg_ram,
    };
    mapper.update_mapping();

    Box::new(mapper)
}

impl Mapper for MMC1 {
    fn prg_rom_read(&mut self, idx: u16) -> &RomBank {
        self.prg_rom.get_bank(idx)
    }

    fn prg_rom_write(&mut self, idx: u16, val: u8) -> &mut RomBank {
        if val & 0b1000_0000 != 0 {
            self.reset();
        } else {
            self.accumulator |= (val & 1) << self.write_counter;
            self.write_counter += 1;

            if self.write_counter == 5 {
                self.do_write(idx);
                self.accumulator = 0;
                self.write_counter = 0;
            }
        }

        self.prg_rom.get_bank_mut(idx)
    }

    fn prg_ram_read(&mut self, idx: u16) -> u8 {
        self.prg_ram.read(prg_ram_addr(idx))
    }

    fn prg_ram_write(&mut self, idx: u16, val: u8) {
        self.prg_ram.write(prg_ram_addr(idx), val);
    }

    fn chr_read(&mut self, idx: u16) -> u8 {
        self.chr_ram[idx as usize]
    }

    fn chr_write(&mut self, idx: u16, val: u8) {
        self.chr_ram[idx as usize] = val;
    }

    fn get_mirroring_table(&self) -> &[u16; 4] {
        self.regs.control.mirroring
    }
}
