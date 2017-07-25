#![feature(test)]
#![feature(plugin)]
#![feature(asm)]
#![feature(naked_functions)]

#![plugin(dynasm)]

#![allow(unused_features)]
#![allow(unknown_lints)]
#![allow(new_without_default)]
#![allow(match_same_arms)]

#[cfg(feature = "jit")]
extern crate dynasmrt;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate quick_error;

pub extern crate sdl2;
extern crate blip_buf;
extern crate memmap;

#[cfg(feature = "vectorize")]
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

use apu::APU;
use cart::Cart;
use cpu::CPU;
use io::IO;
use ppu::PPU;
use std::cell::RefCell;
use std::cell::UnsafeCell;

use std::rc::Rc;

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

    pub fn new_sdl(
        cart: Cart,
        sdl: &sdl2::Sdl,
        event_pump: &Rc<RefCell<sdl2::EventPump>>,
    ) -> EmulatorBuilder {
        EmulatorBuilder {
            cart: cart,

            screen: Box::new(screen::sdl::SDLScreen::new(sdl)),
            audio_out: Box::new(audio::sdl::SDLAudioOut::new(sdl)),
            io: Box::new(io::sdl::SdlIO::new(event_pump.clone())),
        }
    }

    pub fn build(mut self) -> Emulator {
        let dispatcher = Rc::new(UnsafeCell::new(cpu::dispatcher::Dispatcher::new()));
        self.cart.set_dispatcher(dispatcher.clone());
        let cart: Rc<UnsafeCell<Cart>> = Rc::new(UnsafeCell::new(self.cart));
        let ppu = PPU::new(cart.clone(), self.screen);
        let apu = APU::new(self.audio_out);
        let mut cpu = CPU::new(ppu, apu, self.io, cart, dispatcher);
        cpu.init();

        Emulator { cpu: cpu }
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

    #[cfg(feature = "mousepick")]
    pub fn mouse_pick(&self, px_x: i32, px_y: i32) {
        self.cpu.ppu.mouse_pick(px_x, px_y);
    }

    pub fn rendering_enabled(&self) -> bool {
        self.cpu.ppu.rendering_enabled()
    }
}
