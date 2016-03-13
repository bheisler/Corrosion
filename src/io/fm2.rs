use std::fs::File;
use std::io::Result;
use std::io::BufReader;
use std::io::BufRead;
use std::iter::Iterator;

use io::OPEN_BUS;
use io::IO;
use memory::MemSegment;
use util::ShiftRegister8;

pub struct FM2IO {
    iter: Box<Iterator<Item = String>>,
    controller1: ShiftRegister8,
    controller2: ShiftRegister8,
}

impl FM2IO {
    pub fn read(file: String) -> Result<FM2IO> {
        let file = try!(File::open(file));
        let reader = BufReader::new(file);
        let iter = reader.lines()
                         .map(|result| result.unwrap())
                         .skip_while(|line| !line.contains("|"))
                         .skip(1);
        return Ok(FM2IO {
            iter: Box::new(iter),
            controller1: ShiftRegister8::new(0),
            controller2: ShiftRegister8::new(0),
        });
    }
}

fn parse(string: &str) -> u8 {
    string.char_indices()
          .filter(|&(_, c)| c != '.')
          .fold(0u8, |acc, (idx, _)| acc | 1u8 << (7 - idx))
}

impl MemSegment for FM2IO {
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
                    if let Some(line) = self.iter.next() {
                        let mut split = line.split("|")
                                            .skip(1)
                                            .skip(1); //Ignore the commands for now.
                        self.controller1.load(parse(split.next().unwrap()));
                        self.controller2.load(parse(split.next().unwrap()));
                    }
                }
            }
            0x4017 => (),
            x => invalid_address!(x),
        }
    }
}

impl IO for FM2IO {
    fn poll(&mut self) {
        // Do nothing.
    }
}
