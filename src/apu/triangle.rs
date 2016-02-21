//! Contains structures used by the NES's triangle channel.

use apu::Writable;
use apu::components::*;

#[allow(dead_code)] //TODO: Remove this
pub struct Triangle {
    counter: u8,
    timer: u8,
    pub length: Length,
}

#[allow(unused_variables)] //TODO: Remove this
impl Triangle {
    pub fn new() -> Triangle {
        Triangle {
            counter: 0,
            timer: 0,
            length: Length::new(7),
        }
    }

    pub fn length_tick(&mut self) {
        self.length.tick();
    }

    pub fn play(&mut self, from_cyc: u32, to_cyc: u32) {}
}

impl Writable for Triangle {
    fn write(&mut self, idx: u16, val: u8) {
        match idx % 4 {
            0 => self.length.write_halt(val),
            1 => (),
            2 => (),
            3 => self.length.write_counter(val),
            _ => (),
        }
    }
}