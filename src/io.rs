#![allow(unused_variables, dead_code)]

use super::memory::MemSegment;

pub struct IO {
    output: u8,
    controller1: u8,
    controller2: u8,
}

impl IO {
    pub fn new() -> IO {
        IO {
            output: 0,
            controller1: 0,
            controller2: 0,
        }
    }
}

impl MemSegment for IO {
    fn read(&mut self, idx: u16) -> u8 {
        0
    }

    fn write(&mut self, idx: u16, val: u8) {}
}
