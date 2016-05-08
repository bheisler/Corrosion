use ppu::{Color, SCREEN_BUFFER_SIZE};

pub mod sdl;

pub trait Screen {
    fn draw(&mut self, buf: &[Color; SCREEN_BUFFER_SIZE]);
}

pub struct DummyScreen;


impl DummyScreen {
    pub fn new() -> DummyScreen {
        DummyScreen
    }
}

impl Screen for DummyScreen {
    fn draw(&mut self, _: &[Color; SCREEN_BUFFER_SIZE]) {}
}
