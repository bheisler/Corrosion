use std::collections::HashMap;

use io::OPEN_BUS;
use io::IO;
use memory::MemSegment;
use util::ShiftRegister8;

pub struct TestIO {
    frames: u32,
    commands: HashMap<u32, &'static str>,

    controller1: ShiftRegister8,
    controller2: ShiftRegister8,
}

// Commands are in the FM2 RLDUTSBA|RLDUTSBA format minus the commands block at
// the start
impl TestIO {
    pub fn new(commands: HashMap<u32, &'static str>) -> TestIO {
        TestIO {
            frames: 0,
            commands: commands,

            controller1: ShiftRegister8::new(0),
            controller2: ShiftRegister8::new(0),
        }
    }
}

fn parse(string: &str) -> u8 {
    string.char_indices()
        .filter(|&(_, c)| c != '.')
        .fold(0u8, |acc, (idx, _)| acc | 1u8 << (7 - idx))
}

impl MemSegment for TestIO {
    fn read(&mut self, idx: u16) -> u8 {
        match idx {
            0x4016 => OPEN_BUS | self.controller1.shift(),
            0x4017 => OPEN_BUS | self.controller2.shift(),
            x => invalid_address!(x),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        match idx {
            0x4016 => {
                if val & 0x01 != 0 {
                    if let Some(line) = self.commands.get(&self.frames) {
                        let mut split = line.split('|');
                        self.controller1.load(parse(split.next().unwrap()));
                        self.controller2.load(parse(split.next().unwrap()));
                    }
                    self.frames += 1;
                }
            }
            0x4017 => (),
            x => invalid_address!(x),
        }
    }
}

impl IO for TestIO {
    fn poll(&mut self) {
        // Do nothing.
    }
}
