use ppu::{Color, SCREEN_BUFFER_SIZE};

pub mod sdl;

//#[cfg(test)]
pub mod hash_screen;

pub trait Screen {
    fn draw(&mut self, buf: &[Color; SCREEN_BUFFER_SIZE]);
}

pub struct DummyScreen;

impl Default for DummyScreen {
    fn default() -> DummyScreen {
        DummyScreen
    }
}

impl Screen for DummyScreen {
    fn draw(&mut self, _: &[Color; SCREEN_BUFFER_SIZE]) {}
}
