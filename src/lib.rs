#![feature(test)]
#![feature(plugin)]
#![feature(asm)]
#![feature(naked_functions)]

#![plugin(dynasm)]

#![allow(unused_features)]
#![allow(unknown_lints)]
#![allow(new_without_default)]
#![allow(match_same_arms)]

#[cfg(target_arch = "x86_64")]
extern crate dynasmrt;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate quick_error;

#[macro_use]
extern crate nom;

pub extern crate sdl2;
extern crate blip_buf;
extern crate memmap;
extern crate fnv;

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

#[derive(Debug, Clone)]
pub struct Settings {
    pub jit: bool,
    pub graphics_enabled: bool,
    pub sound_enabled: bool,

    // The following will only be used if compiled with the debug_features feature
    pub trace_cpu: bool,
    pub disassemble_functions: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            jit: false,
            graphics_enabled: true,
            sound_enabled: true,

            trace_cpu: false,
            disassemble_functions: false,
        }
    }
}

pub struct EmulatorBuilder {
    cart: Cart,
    settings: Settings,

    pub screen: Box<screen::Screen>,
    pub audio_out: Box<audio::AudioOut>,
    pub io: Box<IO>,
}

impl EmulatorBuilder {
    pub fn new(cart: Cart, settings: Settings) -> EmulatorBuilder {
        EmulatorBuilder {
            cart: cart,
            settings: settings,

            screen: Box::new(screen::DummyScreen::default()),
            audio_out: Box::new(audio::DummyAudioOut),
            io: Box::new(io::DummyIO::Dummy),
        }
    }

    pub fn new_sdl(
        cart: Cart,
        settings: Settings,
        sdl: &sdl2::Sdl,
        event_pump: &Rc<RefCell<sdl2::EventPump>>,
    ) -> EmulatorBuilder {
        let sound_enabled = settings.sound_enabled;
        let mut builder = EmulatorBuilder::new(cart, settings);

        builder.screen = Box::new(screen::sdl::SDLScreen::new(sdl));
        if sound_enabled {
            builder.audio_out = Box::new(audio::sdl::SDLAudioOut::new(sdl));
        }
        builder.io = Box::new(io::sdl::SdlIO::new(event_pump.clone()));

        builder
    }

    pub fn build(self) -> Emulator {
        let settings = Rc::new(self.settings);
        let dispatcher = cpu::dispatcher::Dispatcher::new();
        let cart: Rc<UnsafeCell<Cart>> = Rc::new(UnsafeCell::new(self.cart));
        let ppu = PPU::new(settings.clone(), cart.clone(), self.screen);
        let apu = APU::new(settings.clone(), self.audio_out);
        let mut cpu = CPU::new(settings, ppu, apu, self.io, cart, dispatcher);
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

    #[cfg(feature = "debug_features")]
    pub fn mouse_pick(&self, px_x: i32, px_y: i32) {
        self.cpu.ppu.mouse_pick(px_x, px_y);
    }

    pub fn rendering_enabled(&self) -> bool {
        self.cpu.ppu.rendering_enabled()
    }
}
