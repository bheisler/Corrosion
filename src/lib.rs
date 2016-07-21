#![feature(test)]
#![feature(plugin)]
#![plugin(clippy)]

#![allow(new_without_default)]
#![allow(match_same_arms)]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate quick_error;

pub extern crate sdl2;
extern crate blip_buf;
extern crate memmap;
extern crate simd;

pub mod cart;
pub mod memory;
pub mod mappers;
pub mod ppu;
pub mod apu;
pub mod io;
pub mod cpu;
pub mod screen;
pub mod audio;

mod util;

#[cfg(test)]
mod tests;

#[cfg(feature="cputrace")]
pub mod disasm;

use cart::Cart;
use cpu::CPU;
use memory::CpuMemory;
use apu::APU;
use ppu::PPU;
use io::IO;

use std::rc::Rc;
use std::cell::RefCell;

pub struct EmulatorBuilder {
    pub cart: Cart,

    pub screen: Box<screen::Screen>,
    pub audio_out: Box<audio::AudioOut>,
    pub io: Box<IO>,
}

impl EmulatorBuilder {
    pub fn new(cart: Cart) -> EmulatorBuilder {
        EmulatorBuilder {
            cart: cart,

            screen: Box::new(screen::DummyScreen::default()),
            audio_out: Box::new(audio::DummyAudioOut),
            io: Box::new(io::DummyIO::Dummy),
        }
    }

    pub fn new_sdl(cart: Cart, sdl: &sdl2::Sdl, event_pump: &Rc<RefCell<sdl2::EventPump>>) -> EmulatorBuilder {
        EmulatorBuilder {
            cart: cart,

            screen: Box::new(screen::sdl::SDLScreen::new(sdl)),
            audio_out: Box::new(audio::sdl::SDLAudioOut::new(sdl)),
            io: Box::new(io::sdl::SdlIO::new(event_pump.clone())),
        }
    }

    pub fn build(self) -> Emulator {
        let cart: Rc<RefCell<Cart>> = Rc::new(RefCell::new(self.cart));
        let ppu = PPU::new(cart.clone(), self.screen);
        let apu = APU::new(self.audio_out);
        let mem = CpuMemory::new(ppu, apu, self.io, cart);
        let mut cpu = CPU::new(mem);
        cpu.init();

        Emulator{ cpu: cpu }
    }
}

pub struct Emulator {
    cpu: CPU,
}

impl Emulator {
    pub fn run_frame(&mut self) {
        self.cpu.run_frame();
    }

    pub fn halted(&self) -> bool {
        self.cpu.halted()
    }

    #[cfg(feature="mousepick")]
    pub fn mouse_pick(&self, px_x: i32, px_y: i32) {
        self.cpu.mem.ppu.mouse_pick(px_x, px_y);
    }

    pub fn rendering_enabled(&self) -> bool {
        self.cpu.mem.ppu.rendering_enabled()
    }
}
