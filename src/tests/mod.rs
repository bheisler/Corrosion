mod hash_screen;
mod test_io;

use std::collections::HashMap;
use std::path::Path;

#[test]
fn verify_completes_nestest() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let mut commands: HashMap<u32, &'static str> = HashMap::new();

    //Run the main tests
    commands.insert(10, "....T...|........");
    hashes.insert(35, "2bfe5ffe2fae65fa730c04735a3b25115c5fb65e");

    //Switch to the unofficial opcode tests and run them
    commands.insert(40, ".....S..|........");
    commands.insert(45, "....T...|........");
    hashes.insert(65, "0b6895e6ff0e8be76e805a067be6ebec89e7d6ad");

    run_system_test(70,
                    Path::new("nes-test-roms/other/nestest.nes"),
                    hashes,
                    commands);
}

fn run_system_test(frames: u32,
                   file_name: &Path,
                   hashes: HashMap<u32, &'static str>,
                   commands: HashMap<u32, &'static str>) {

    let cart = ::cart::Cart::read(file_name).expect("Failed to read ROM File");
    let mut builder = ::EmulatorBuilder::new(cart);
    builder.io = Box::new(test_io::TestIO::new(commands));
    builder.screen = Box::new(hash_screen::HashVerifier::new(hashes));
    builder.screen = Box::new(hash_screen::HashPrinter::new(builder.screen));

    let mut emulator = builder.build();

    for _ in 0..frames {
        assert!(!emulator.halted());
        emulator.run_frame();
    }
}
