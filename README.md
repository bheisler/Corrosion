An NES emulator written in Rust as a hobby project. It only supports Mappers 0 and 1 at this point, but that's enough to play Donkey Kong, Super Mario Bros and Legend of Zelda. It also has a working (though rudimentary) just-in-time compiler targeting x86_64 machine code with [dynasm-rs](https://github.com/CensoredUsername/dynasm-rs). The JIT compiler is currently not well-optimized and has difficulty dealing with heavy bankswitching (eg. Legend of Zelda runs slower with JIT than without due to excessive recompilation) but I hope to improve on that in the future.

### Building

In order to build corrosion, you will require a recent nightly version of Rust. Nightly is currently required in order to use dynasm-rs. You will also need to install [sdl2](https://github.com/AngryLawyer/rust-sdl2). Once ready, building corrosion is as simple as running `cargo build --release` in the `app` directory. There are a number of additional features which can be compiled into the binary like so:

    cargo build --release --features cputrace

For a complete list of which features are available, check Cargo.toml, though not all combinations are tested, so experiment at your own risk.
