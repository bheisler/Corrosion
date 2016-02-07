pub mod sdl;

use ::apu::OutputBuffer;

pub trait AudioOut {
    fn play(&mut self, buffer: &OutputBuffer);
}

pub struct DummyAudioOut;

impl AudioOut for DummyAudioOut {
    fn play(&mut self, _: &OutputBuffer) {}
}
