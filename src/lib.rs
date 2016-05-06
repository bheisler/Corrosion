#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate quick_error;

extern crate sdl2;
extern crate stopwatch;
extern crate blip_buf;
extern crate memmap;

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

#[cfg(feature="cputrace")]
pub mod disasm;

use cart::Cart;
use cpu::CPU;
use memory::CpuMemory;
use apu::APU;
use ppu::PPU;
use io::IO;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::Sdl;

use std::rc::Rc;
use std::cell::RefCell;

use stopwatch::Stopwatch;

fn pump_events(pump: &Rc<RefCell<EventPump>>) -> bool {
    for event in pump.borrow_mut().poll_iter() {
        match event {
            Event::Quit {..} => return true,
            _ => (),
        }
    }
    false
}

fn get_movie_file() -> Option<String> {
    return std::env::args()
               .skip_while(|arg| arg != "--movie")
               .skip(1)
               .next();
}

#[cfg(feature="mousepick")]
fn mouse_pick(sdl: &Sdl, cpu: &CPU ) {
    let (_, scr_x, scr_y) = sdl.mouse().mouse_state();
    let (px_x, px_y) = (scr_x / 3, scr_y / 3); //Should get this from the screen, but eh.
    cpu.mem.ppu.mouse_pick(px_x, px_y);
}

#[cfg(not(feature="mousepick"))]
fn mouse_pick(_: &Sdl, _: &CPU ) {

}

pub fn start_emulator(cart: Cart) {
    let sdl = sdl2::init().unwrap();
    let screen = screen::sdl::SDLScreen::new(&sdl);
    let audio_out = audio::sdl::SDLAudioOut::new(&sdl);
    let event_pump = Rc::new(RefCell::new(sdl.event_pump().unwrap()));

    let cart: Rc<RefCell<Cart>> = Rc::new(RefCell::new(cart));
    let ppu = PPU::new(cart.clone(), Box::new(screen));
    let apu = APU::new(Box::new(audio_out));
    let io: Box<IO> = if let Some(file) = get_movie_file() {
        let fm2io = io::fm2::FM2IO::read(file).unwrap();
        Box::new(fm2io)
    } else {
        Box::new(io::sdl::SdlIO::new(event_pump.clone()))
    };
    let mem = CpuMemory::new(ppu, apu, io, cart);
    let mut cpu = CPU::new(mem);
    cpu.init();

    let mut stopwatch = Stopwatch::start_new();
    let smoothing = 0.9;
    let mut avg_frame_time = 0.0f64;
    loop {
        if pump_events(&event_pump) || cpu.halted() {
            break;
        }
        cpu.run_frame();
        let current = stopwatch.elapsed().num_nanoseconds().unwrap() as f64;
        avg_frame_time = (avg_frame_time * smoothing) + (current * (1.0 - smoothing));

        mouse_pick(&sdl, &cpu);

        //println!("Frames per second:{:.*}", 2, 1000000000.0 / avg_frame_time);
        stopwatch.restart();
    }
}
