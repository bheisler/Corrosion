pub mod sdl;

use ::apu::Sample;

pub trait AudioOut {
    fn play(&mut self, buffer: &[Sample]);
    fn sample_rate(&self) -> f64;
}

pub struct DummyAudioOut;

impl AudioOut for DummyAudioOut {
    fn play(&mut self, _: &[Sample]) {}
    fn sample_rate(&self) -> f64 { 44100.0 }
}
