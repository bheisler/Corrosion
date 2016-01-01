extern crate nes_emulator;

use std::env;
use std::path::Path;
use nes_emulator::cart::Cart;
use nes_emulator::start_emulator;

fn main() {
    let args = env::args();
    let file_name = args.skip(1).next().expect("No ROM file provided.");
    let path = Path::new(&file_name);
    let cart = Cart::read(&path).expect("Failed to read ROM File");
    start_emulator(cart);
}
