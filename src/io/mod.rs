pub mod sdl;
pub mod fm2;

use super::memory::MemSegment;

///Some bits of the controller reads return open bus garbage. Since the last byte on the bus is
///almost always 0x40, we can just use that as a constant for now.
const OPEN_BUS: u8 = 0x40;

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
