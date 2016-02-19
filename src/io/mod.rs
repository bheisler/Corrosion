pub mod sdl;

use super::memory::MemSegment;

pub trait IO : MemSegment {
    fn poll(&mut self);
}

pub enum DummyIO {
    Dummy,
}

impl DummyIO {
    pub fn new() -> DummyIO {
        DummyIO::Dummy
    }
}

impl MemSegment for DummyIO {
    fn read(&mut self, _: u16) -> u8 {
        0
    }

    fn write(&mut self, _: u16, _: u8) {
        ()
    }
}

impl IO for DummyIO {
    fn poll(&mut self) {
        ()
    }
}
