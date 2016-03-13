//! Contains structures used by the NES's DMC channel.

use apu::Writable;

#[allow(dead_code)]
pub struct DMC {
    freq: u8,
    direct: u8,
    sample_addr: u8,
    sample_length: u8,
}

#[allow(unused_variables)]
impl DMC {
    pub fn new() -> DMC {
        DMC {
            freq: 0,
            direct: 0,
            sample_addr: 0,
            sample_length: 0,
        }
    }

    pub fn play(&mut self, from_cyc: u32, to_cyc: u32) {}
}

#[allow(unused_variables)]
impl Writable for DMC {
    fn write(&mut self, idx: u16, val: u8) {}
}
