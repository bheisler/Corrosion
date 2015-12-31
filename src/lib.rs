#[macro_use]
extern crate bitflags;

pub mod rom;
pub mod memory;
pub mod mappers;
pub mod ppu;
pub mod apu;
pub mod io;
pub mod cpu;

#[cfg(feature="cputrace")]
pub mod disasm;

use rom::Rom;
use mappers::Mapper;
use cpu::CPU;
use memory::CpuMemory;
use io::IO;
use apu::APU;
use ppu::{PPU, PPUMemory};

use std::rc::Rc;
use std::cell::RefCell;

pub fn start_emulator(rom: Rom) {
    let mapper = Mapper::new(rom.mapper() as u16, rom.prg_rom, rom.chr_rom, rom.prg_ram);
    let mapper: Rc<RefCell<Box<Mapper>>> = Rc::new(RefCell::new(mapper));
    let ppu_mem = PPUMemory::new(mapper.clone());
    let ppu = PPU::new(ppu_mem);
    let apu = APU::new();
    let io = IO::new();
    let mem = CpuMemory::new(ppu, apu, io, mapper);
    let mut cpu = CPU::new(mem);
    cpu.init();

    loop {
        cpu.step();
    }
}
