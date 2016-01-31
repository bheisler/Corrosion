#![allow(unused_variables, dead_code)]

use super::memory::MemSegment;

///Some bits of the controller reads return open bus garbage. Since the last byte on the bus is
///almost always 0x40, we can just use that as a constant for now.
const OPEN_BUS: u8 = 0x40;

pub struct IO {
    strobe: bool,
    controller1: u8,
    controller2: u8,
}

impl IO {
    pub fn new() -> IO {
        IO {
            strobe: false,
            controller1: 0,
            controller2: 0,
        }
    }
}

fn shift_reg(val: &mut u8) -> u8 {
    let result = *val & 0x01;
    *val = *val >> 1;
    result
}

impl MemSegment for IO {
    fn read(&mut self, idx: u16) -> u8 {
        match idx {
            0x4016 => OPEN_BUS | shift_reg( &mut self.controller1 ),
            0x4017 => OPEN_BUS | shift_reg( &mut self.controller2 ),
            x => invalid_address!(x),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        match idx {
            0x4016 => self.strobe = val & 0x01 != 0,
            0x4017 => (),
            x => invalid_address!(x),
        }
    }
}
