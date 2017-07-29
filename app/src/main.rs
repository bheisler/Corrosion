extern crate corrosion;
extern crate stopwatch;
extern crate config;


use config::{Config, File};

use corrosion::{Emulator, EmulatorBuilder, Settings};
use corrosion::cart::Cart;
use corrosion::sdl2::EventPump;
use corrosion::sdl2::event::Event;
use std::cell::RefCell;
use std::env;
use std::path::Path;

use std::rc::Rc;
use stopwatch::Stopwatch;

fn main() {
    let args = env::args();
    let file_name = args.skip(1).next().expect("No ROM file provided.");
    let path = Path::new(&file_name);
    let cart = Cart::read(&path).expect("Failed to read ROM File");
    let config = load_config();
    start_emulator(cart, config);
}

fn load_config() -> Config {
    let mut s = Config::new();
    s.merge(File::with_name("config/default").required(false))
        .expect("Failed to read config file");
    s
}

fn make_emulator_settings(config: &Config) -> Settings {
    let defaults : Settings = Default::default();
    Settings {
        trace_cpu: config.get_bool("debug.trace_cpu").unwrap_or(defaults.trace_cpu),
        disassemble_functions: config.get_bool("debug.disassemble_functions").unwrap_or(defaults.disassemble_functions),
    }
}

#[cfg(feature = "debug_features")]
fn mouse_pick(event_pump: &Rc<RefCell<EventPump>>, emulator: &Emulator) {
    let mouse_state = event_pump.borrow().mouse_state();
    let (px_x, px_y) = (mouse_state.x() / 3, mouse_state.y() / 3); // Should get this from the screen, but eh.
    emulator.mouse_pick(px_x, px_y);
}

#[cfg(not(feature = "debug_features"))]
fn mouse_pick(_: &Rc<RefCell<EventPump>>, _: &Emulator) {}

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

fn start_emulator(cart: Cart, config: Config) {
    let sdl = corrosion::sdl2::init().unwrap();
    let event_pump = Rc::new(RefCell::new(sdl.event_pump().unwrap()));

    let mut builder = EmulatorBuilder::new_sdl(cart, make_emulator_settings(&config), &sdl, &event_pump);

    if let Some(file) = get_movie_file() {
        let fm2io = corrosion::io::fm2::FM2IO::read(file).unwrap();
        builder.io = Box::new(fm2io)
    }

    let mut emulator = builder.build();

    let mut stopwatch = Stopwatch::start_new();
    let smoothing = 0.9;
    let mut avg_frame_time = 0.0f64;
    let mousepick_enabled = config.get_bool("debug.mousepick").unwrap_or(false);
    loop {
        if pump_events(&event_pump) || emulator.halted() {
            break;
        }
        emulator.run_frame();
        let current = stopwatch.elapsed().num_nanoseconds().unwrap() as f64;
        avg_frame_time = (avg_frame_time * smoothing) + (current * (1.0 - smoothing));

        if mousepick_enabled {
            mouse_pick(&event_pump, &emulator);
        }

        // println!("Frames per second:{:.*}", 2, 1000000000.0 / avg_frame_time);
        stopwatch.restart();
    }
}
