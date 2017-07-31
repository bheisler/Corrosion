extern crate test;

use self::test::Bencher;
use Settings;
use std::collections::HashMap;
use std::path::Path;
use tests::test_io;

#[bench]
fn bench_sprite(b: &mut Bencher) {
    run_benchmark(
        b,
        Path::new("nes-test-roms/other/SPRITE.NES"),
        HashMap::new(),
        Default::default(),
    );
}

#[bench]
fn bench_cpu_sprite_jit(b: &mut Bencher) {
    run_benchmark(
        b,
        Path::new("nes-test-roms/other/SPRITE.NES"),
        HashMap::new(),
        cpu_benchmark_settings(true),
    );
}

#[bench]
fn bench_cpu_sprite_no_jit(b: &mut Bencher) {
    run_benchmark(
        b,
        Path::new("nes-test-roms/other/SPRITE.NES"),
        HashMap::new(),
        cpu_benchmark_settings(false),
    );
}

#[bench]
fn bench_blocks(b: &mut Bencher) {
    run_benchmark(
        b,
        Path::new("nes-test-roms/other/BLOCKS.NES"),
        HashMap::new(),
        Default::default(),
    );
}

#[bench]
fn bench_cpu_blocks_jit(b: &mut Bencher) {
    run_benchmark(
        b,
        Path::new("nes-test-roms/other/BLOCKS.NES"),
        HashMap::new(),
        cpu_benchmark_settings(true),
    );
}

#[bench]
fn bench_cpu_blocks_no_jit(b: &mut Bencher) {
    run_benchmark(
        b,
        Path::new("nes-test-roms/other/BLOCKS.NES"),
        HashMap::new(),
        cpu_benchmark_settings(false),
    );
}

pub fn cpu_benchmark_settings(jit: bool) -> Settings {
    Settings {
        jit: jit,
        graphics_enabled: false,
        sound_enabled: false,
        ..Default::default()
    }
}

pub fn run_benchmark(
    bencher: &mut Bencher,
    file_name: &Path,
    commands: HashMap<u32, &'static str>,
    settings: Settings,
) {

    let cart = ::cart::Cart::read(file_name).expect("Failed to read ROM File");
    let mut builder = ::EmulatorBuilder::new(cart, settings);
    builder.io = Box::new(test_io::TestIO::new(commands));

    let mut emulator = builder.build();

    while !emulator.rendering_enabled() {
        assert!(!emulator.halted());
        emulator.run_frame();
    }

    bencher.iter(|| {
        assert!(!emulator.halted());
        emulator.run_frame();
    });
}
