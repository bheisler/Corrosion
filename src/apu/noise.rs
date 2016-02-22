//! Contains structures used by the NES's noise channel.

use apu::Writable;
use apu::components::*;

#[allow(dead_code)] //TODO: Remove this
pub struct Noise {
    envelope: Envelope,
    mode: u8,
    pub length: Length,
}

#[allow(unused_variables)] //TODO: Remove this
impl Noise {
    pub fn new() -> Noise {
        Noise {
            envelope: Envelope::new(),
            mode: 0,
            length: Length::new(5),
        }
    }

    pub fn length_tick(&mut self) {
        self.length.tick();
    }

    pub fn envelope_tick(&mut self) {
        self.envelope.tick();
    }

    pub fn play(&mut self, from_cyc: u32, to_cyc: u32) {}
}

impl Writable for Noise {
    fn write(&mut self, idx: u16, val: u8) {
        match idx % 4 {
            0 => {
                self.length.write_halt(val);
                self.envelope.write(val);
            }
            1 => (),
            2 => (),
            3 => self.length.write_counter(val),
            _ => (),
        }
    }
}
