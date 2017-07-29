extern crate test;

use std::collections::HashMap;
use std::path::Path;
use self::test::Bencher;
use tests::test_io;

#[bench]
fn bench_sprite(b: &mut Bencher) {
    run_benchmark(b,
                  Path::new("nes-test-roms/other/SPRITE.NES"),
                  HashMap::new());
}

#[bench]
fn bench_blocks(b: &mut Bencher) {
    run_benchmark(b,
                  Path::new("nes-test-roms/other/BLOCKS.NES"),
                  HashMap::new());
}

fn run_benchmark(bencher: &mut Bencher, file_name: &Path, commands: HashMap<u32, &'static str>) {

    let cart = ::cart::Cart::read(file_name).expect("Failed to read ROM File");
    let mut builder = ::EmulatorBuilder::new(cart, Default::default());
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
