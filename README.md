An NES emulator written in Rust as a hobby project. It only supports Mappers 0 and 1 at this point, but that's enough to play Donkey Kong, Super Mario Bros and Legend of Zelda. It also has a working (though rudimentary) just-in-time compiler targeting x86_64 machine code with [dynasm-rs](https://github.com/CensoredUsername/dynasm-rs). The JIT compiler is currently not well-optimized and has difficulty dealing with heavy bankswitching (eg. Legend of Zelda runs slower with JIT than without due to excessive recompilation) but I hope to improve on that in the future.

### Building

In order to build corrosion, you will require a recent nightly version of Rust. Nightly is currently required in order to use dynasm-rs. You will also need to install [sdl2](https://github.com/AngryLawyer/rust-sdl2). Once ready, building corrosion is as simple as running `cargo build --release` in the `app` directory.

There are a number of additional debug features. These must be compiled into the executable like so:

    cargo build --release --features debug_features

Once compiled, they can be enabled by setting the flags in app/config/default.toml. See the config file for supported features.
