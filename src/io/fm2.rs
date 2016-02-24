use std::fs::File;
use std::io::Result;

use io::OPEN_BUS;
use io::IO;
use memory::MemSegment;
use util::ShiftRegister8;

pub struct FM2IO {
    x: u8,
}

impl FM2IO {
    pub fn read(file: String) -> Result<FM2IO> {
        return Ok( FM2IO {
                x: 0
            } );
    }
}

impl MemSegment for FM2IO {
        fn read(&mut self, idx: u16) -> u8 {
        match idx {
            0x4016 => OPEN_BUS,// | self.controller1.shift(),
            0x4017 => OPEN_BUS,// | self.controller2.shift(),
            x => invalid_address!(x),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        match idx {
            0x4016 => (),
            0x4017 => (),
            x => invalid_address!(x),
        }
    }
}

impl IO for FM2IO {
    fn poll(&mut self) {
        //TODO
    }
}