extern crate corrosion;
extern crate stopwatch;

use std::env;
use std::path::Path;
use corrosion::cart::Cart;
use corrosion::sdl2::event::Event;
use corrosion::sdl2::EventPump;
use corrosion::sdl2::Sdl;
use stopwatch::Stopwatch;

use corrosion::{Emulator, EmulatorBuilder};

use std::rc::Rc;
use std::cell::RefCell;


fn main() {
    let args = env::args();
    let file_name = args.skip(1).next().expect("No ROM file provided.");
    let path = Path::new(&file_name);
    let cart = Cart::read(&path).expect("Failed to read ROM File");
    start_emulator(cart);
}

#[cfg(feature="mousepick")]
fn mouse_pick(sdl: &Sdl, emulator: &Emulator) {
    let (_, scr_x, scr_y) = sdl.mouse().mouse_state();
    let (px_x, px_y) = (scr_x / 3, scr_y / 3); //Should get this from the screen, but eh.
    emulator.mouse_pick(px_x, px_y);
}

#[cfg(not(feature="mousepick"))]
fn mouse_pick(_: &Sdl, _: &Emulator) {}

fn pump_events(pump: &Rc<RefCell<EventPump>>) -> bool {
    for event in pump.borrow_mut().poll_iter() {
        if let Event::Quit { .. } = event {
            return true;
        }
    }
    false
}

fn get_movie_file() -> Option<String> {
    std::env::args()
        .skip_while(|arg| arg != "--movie")
        .skip(1)
        .next()
}

fn start_emulator(cart: Cart) {
    let sdl = corrosion::sdl2::init().unwrap();
    let event_pump = Rc::new(RefCell::new(sdl.event_pump().unwrap()));

    let mut builder = EmulatorBuilder::new_sdl(cart, &sdl, &event_pump);

    if let Some(file) = get_movie_file() {
        let fm2io = corrosion::io::fm2::FM2IO::read(file).unwrap();
        builder.io = Box::new(fm2io)
    }

    let mut emulator = builder.build();

    let mut stopwatch = Stopwatch::start_new();
    let smoothing = 0.9;
    let mut avg_frame_time = 0.0f64;
    loop {
        if pump_events(&event_pump) || emulator.halted() {
            break;
        }
        emulator.run_frame();
        let current = stopwatch.elapsed().num_nanoseconds().unwrap() as f64;
        avg_frame_time = (avg_frame_time * smoothing) + (current * (1.0 - smoothing));

        mouse_pick(&sdl, &emulator);

        // println!("Frames per second:{:.*}", 2, 1000000000.0 / avg_frame_time);
        stopwatch.restart();
    }
}
