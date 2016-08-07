use super::*;
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
    prg_bank: u8,
}

struct MMC1 {
    regs: Regs,

    accumulator: u8,
    write_counter: u8,

    prg_rom: Box<[u8]>,
    chr_ram: Box<[u8]>,
    prg_ram: Box<MemSegment>,
}

impl MMC1 {
    fn first_bank(&self) -> u8 {
        match self.regs.control.mode {
            PrgMode::Switch32Kb => self.regs.prg_bank & 0b0001_1110,
            PrgMode::FixFirst => 0,
            PrgMode::FixLast => self.regs.prg_bank,
        }
    }

    fn second_bank(&self) -> u8 {
        match self.regs.control.mode {
            PrgMode::Switch32Kb => self.regs.prg_bank | 1,
            PrgMode::FixFirst => self.regs.prg_bank,
            PrgMode::FixLast => (self.prg_rom.len() / 0x4000) as u8 - 1,
        }
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

    Box::new(MMC1 {
        regs: Regs {
            control: Ctrl {
                mode: PrgMode::FixLast,
                mirroring: super::standard_mapping_tables( ScreenMode::OneScreenLow ),
            },
            chr_0: 0,
            chr_1: 0,
            prg_bank: 0,
        },
        accumulator: 0,
        write_counter: 0,
        prg_rom: params.prg_rom.into_boxed_slice(),
        chr_ram: chr_ram,
        prg_ram: prg_ram,
    })
}

impl Mapper for MMC1 {
    fn prg_read(&mut self, idx: u16) -> u8 {
        let bank = match idx {
            0x6000...0x7FFF => return self.prg_ram.read(prg_ram_addr(idx)),
            0x8000...0xBFFF => self.first_bank(),
            0xC000...0xFFFF => self.second_bank(),
            x => invalid_address!(x),
        };
        let address = (bank as usize * 0x4000) | (idx as usize & 0x3FFF);
        self.prg_rom[address]
    }

    fn prg_write(&mut self, idx: u16, val: u8) {
        if 0x6000 <= idx && idx <= 0x7FFF {
            self.prg_ram.write(prg_ram_addr(idx), val);
            return;
        }

        if val & 0b1000_0000 != 0 {
            self.accumulator = 0;
            self.write_counter = 0;
            self.regs.control = Ctrl {
                mode: PrgMode::FixLast,
                mirroring: super::standard_mapping_tables( ScreenMode::OneScreenLow ),
            };
            return;
        }

        self.accumulator |= (val & 1) << self.write_counter;
        self.write_counter += 1;

        if self.write_counter == 5 {

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
                        mirroring: super::standard_mapping_tables( mirroring ),
                    };
                }
                0xA000...0xBFFF => self.regs.chr_0 = self.accumulator,
                0xC000...0xDFFF => self.regs.chr_1 = self.accumulator,
                0xE000...0xFFFF => self.regs.prg_bank = self.accumulator,
                x => invalid_address!(x),
            }
            self.accumulator = 0;
            self.write_counter = 0;
        }
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
