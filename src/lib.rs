#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate quick_error;

pub mod cart;
pub mod memory;
pub mod mappers;
pub mod ppu;
pub mod apu;
pub mod io;
pub mod cpu;

#[cfg(feature="cputrace")]
pub mod disasm;

use cart::Cart;
use cpu::CPU;
use memory::CpuMemory;
use io::IO;
use apu::APU;
use ppu::PPU;

use std::rc::Rc;
use std::cell::RefCell;

pub fn start_emulator(cart: Cart) {
    let cart: Rc<RefCell<Cart>> = Rc::new(RefCell::new(cart));
    let ppu = PPU::new(cart.clone());
    let apu = APU::new();
    let io = IO::new();
    let mem = CpuMemory::new(ppu, apu, io, cart);
    let mut cpu = CPU::new(mem);
    cpu.init();

    loop {
        cpu.step();
    }
}
