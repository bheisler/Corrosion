mod hash_screen;
mod test_io;
mod bench;

use Settings;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn verify_completes_nestest() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let mut commands: HashMap<u32, &'static str> = HashMap::new();

    // Run the main tests
    commands.insert(10, "....T...|........");
    hashes.insert(35, "2bfe5ffe2fae65fa730c04735a3b25115c5fb65e");

    // Switch to the unofficial opcode tests and run them
    commands.insert(40, ".....S..|........");
    commands.insert(45, "....T...|........");
    hashes.insert(65, "0b6895e6ff0e8be76e805a067be6ebec89e7d6ad");

    run_system_test(
        70,
        Path::new("nes-test-roms/other/nestest.nes"),
        hashes,
        commands,
    );
}

#[test]
fn blargg_apu_test_len_ctr() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(18, "ea9ac1696a5cec416f0a9f34c052815ca59850d5");

    run_system_test(
        19,
        Path::new("nes-test-roms/apu_test/rom_singles/1-len_ctr.nes"),
        hashes,
        commands,
    );
}

#[test]
fn blargg_apu_test_len_table() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(13, "90a61bd003c5794713aa5f207b9b70c8862d892b");

    run_system_test(
        14,
        Path::new("nes-test-roms/apu_test/rom_singles/2-len_table.nes"),
        hashes,
        commands,
    );
}

#[test]
fn blargg_apu_test_irq_flag() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(18, "09e4ad012c8fddfd8e3b4cc6d1b395c5062768c2");

    run_system_test(
        19,
        Path::new("nes-test-roms/apu_test/rom_singles/3-irq_flag.nes"),
        hashes,
        commands,
    );
}

#[test]
fn blargg_ppu_test_palette_ram() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(18, "cb15f68f631c1d409beefb775bcff990286096fb");

    run_system_test(
        19,
        Path::new("nes-test-roms/blargg_ppu_tests_2005.09.15b/palette_ram.nes"),
        hashes,
        commands,
    );
}

#[test]
fn blargg_ppu_test_sprite_ram() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(18, "cb15f68f631c1d409beefb775bcff990286096fb");

    run_system_test(
        19,
        Path::new("nes-test-roms/blargg_ppu_tests_2005.09.15b/sprite_ram.nes"),
        hashes,
        commands,
    );
}

#[test]
fn blargg_ppu_test_vram_access() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(18, "cb15f68f631c1d409beefb775bcff990286096fb");

    run_system_test(
        19,
        Path::new("nes-test-roms/blargg_ppu_tests_2005.09.15b/vram_access.nes"),
        hashes,
        commands,
    );
}

#[test]
fn oam_read() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(27, "cc2447362cceb400803a18c2e4b5d5d4e4aa2ea7");

    run_system_test(
        28,
        Path::new("nes-test-roms/oam_read/oam_read.nes"),
        hashes,
        commands,
    );
}

#[test]
fn sprite_hit_basics() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(33, "1437c48bb22dd3be0d37449171d2120e13877326");

    run_system_test(
        33,
        Path::new("nes-test-roms/sprite_hit_tests_2005.10.05/01.basics.nes"),
        hashes,
        commands,
    );
}

#[test]
fn sprite_hit_alignment() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(31, "33815f5682dda683d1a9fe7495f6358c0e741a9d");

    run_system_test(
        32,
        Path::new("nes-test-roms/sprite_hit_tests_2005.10.05/02.alignment.nes"),
        hashes,
        commands,
    );
}

#[test]
fn sprite_hit_corners() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(21, "760203cab0bc4df16bda48438f67a91e8a152fb9");

    run_system_test(
        22,
        Path::new("nes-test-roms/sprite_hit_tests_2005.10.05/03.corners.nes"),
        hashes,
        commands,
    );
}

#[test]
fn sprite_hit_flip() {
    let mut hashes: HashMap<u32, &'static str> = HashMap::new();
    let commands: HashMap<u32, &'static str> = HashMap::new();

    hashes.insert(21, "e16e43e5efdeacfd999a8ea031fa5058ec202f96");

    run_system_test(
        22,
        Path::new("nes-test-roms/sprite_hit_tests_2005.10.05/04.flip.nes"),
        hashes,
        commands,
    );
}

fn run_system_test(
    frames: u32,
    file_name: &Path,
    hashes: HashMap<u32, &'static str>,
    commands: HashMap<u32, &'static str>,
) {

    let cart = ::cart::Cart::read(file_name).expect("Failed to read ROM File");
    let settings = Settings {
        jit: true,
        ..Default::default()
    };
    let mut builder = ::EmulatorBuilder::new(cart, settings);
    builder.io = Box::new(test_io::TestIO::new(commands));
    builder.screen = Box::new(hash_screen::HashVerifier::new(hashes));
    builder.screen = Box::new(hash_screen::HashPrinter::new(builder.screen));

    let mut emulator = builder.build();

    for _ in 0..frames {
        assert!(!emulator.halted());
        emulator.run_frame();
    }
}
