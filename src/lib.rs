#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate quick_error;
extern crate sdl2;

pub mod cart;
pub mod memory;
pub mod mappers;
pub mod ppu;
pub mod apu;
pub mod io;
pub mod cpu;
pub mod screen;

#[cfg(feature="cputrace")]
pub mod disasm;

use cart::Cart;
use cpu::CPU;
use memory::CpuMemory;
use io::IO;
use apu::APU;
use ppu::PPU;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use sdl2::event::Event;

use std::rc::Rc;
use std::cell::RefCell;

fn pump_events(pump: &mut EventPump) -> bool {
    for event in pump.poll_iter() {
        match event {
            Event::Quit {..} => return true,
            _ => (),
        }
    }
    false
}

pub fn start_emulator(cart: Cart) {
    let sdl = sdl2::init().unwrap();
    let screen = screen::sdl::SDLScreen::new(&sdl);
    let mut event_pump = sdl.event_pump().unwrap();

    let cart: Rc<RefCell<Cart>> = Rc::new(RefCell::new(cart));
    let ppu = PPU::new(cart.clone(), Box::new(screen));
    let apu = APU::new();
    let io = IO::new();
    let mem = CpuMemory::new(ppu, apu, io, cart);
    let mut cpu = CPU::new(mem);
    cpu.init();

    loop {
        if pump_events(&mut event_pump) {
            break;
        }
        cpu.step();
    }
}
