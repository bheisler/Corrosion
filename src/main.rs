mod rom;

use std::env;
use std::path::Path;

fn main() {
    let args = env::args();
    let rom_name = args.skip(1).next().expect("No ROM file provided.");
    let path = Path::new(&rom_name);
    let rom = rom::Rom::read(&path);
}
