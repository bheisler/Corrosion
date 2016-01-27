#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate quick_error;
extern crate sdl2;
extern crate stopwatch;

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
use sdl2::EventPump;
use sdl2::event::Event;

use std::rc::Rc;
use std::cell::RefCell;

use stopwatch::Stopwatch;

fn pump_events(pump: &mut EventPump) -> bool {
    for event in pump.poll_iter() {
        match event {
            Event::Quit {..} => return true,
            _ => (),
        }
    }
    false
}

fn run_frame(cpu: &mut CPU, ppu: &Rc<RefCell<PPU>>) {
    loop {
        let cycle = cpu.cycle();
        let nmi = ppu.borrow_mut().run_to(cycle);
        let frame_end = nmi == ::ppu::StepResult::NMI;
        if frame_end {
            cpu.nmi();
        }
        cpu.step();
        if frame_end {
            break;
        }
    }
}

pub fn start_emulator(cart: Cart) {
    let sdl = sdl2::init().unwrap();
    let screen = screen::sdl::SDLScreen::new(&sdl);
    let mut event_pump = sdl.event_pump().unwrap();

    let cart: Rc<RefCell<Cart>> = Rc::new(RefCell::new(cart));
    let ppu = PPU::new(cart.clone(), Box::new(screen));
    let ppu = Rc::new(RefCell::new(ppu));
    let apu = APU::new();
    let io = IO::new();
    let mem = CpuMemory::new(ppu.clone(), apu, io, cart);
    let mut cpu = CPU::new(mem);
    cpu.init();

    let mut stopwatch = Stopwatch::start_new();
    let smoothing = 0.9;
    let mut avg_frame_time = 0.0f64;
    loop {
        if pump_events(&mut event_pump) || cpu.halted() {
            break;
        }
        run_frame(&mut cpu, &ppu);
        let current = stopwatch.elapsed().num_nanoseconds().unwrap() as f64;
        avg_frame_time = (avg_frame_time * smoothing) + (current * (1.0 - smoothing));
        println!("Frames per second:{:.*}", 2, 1000000000.0 / avg_frame_time);
        stopwatch.restart();
    }
}
