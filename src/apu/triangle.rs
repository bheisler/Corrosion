//! Contains structures used by the NES's triangle channel.

use apu::Writable;
use apu::components::*;

struct LinearCounter {
    control: bool,
    reload: bool,
    value: u8,
    counter: u8,
}

impl LinearCounter {
    fn new() -> LinearCounter {
        LinearCounter {
            control: false,
            reload: false,
            value: 0,
            counter: 0,
        }
    }
    
    fn write(&mut self, val: u8) {
        self.value = val & 0b0111_1111;
        self.control = val & 0b1000_000 != 0;
    }
    
    fn tick(&mut self) {
        if self.reload {
            self.counter = self.value;
        }
        else {
            self.counter = self.counter.saturating_sub(1);
        }
        
        if !self.control {
            self.reload = false;
        }
    }
}

pub struct Triangle {
    counter: LinearCounter,
    timer: Timer,
    pub length: Length,
}

#[allow(unused_variables)] //TODO: Remove this
impl Triangle {
    pub fn new() -> Triangle {
        Triangle {
            counter: LinearCounter::new(),
            timer: Timer::new(1),
            length: Length::new(7),
        }
    }

    pub fn length_tick(&mut self) {
        self.length.tick();
    }
    
    pub fn envelope_tick(&mut self) {
        self.counter.tick();
    }

    pub fn play(&mut self, from_cyc: u32, to_cyc: u32) {}
}

impl Writable for Triangle {
    fn write(&mut self, idx: u16, val: u8) {
        match idx % 4 {
            0 => {
                self.length.write_halt(val);
                self.counter.write(val);
            },
            1 => (),
            2 => self.timer.write_low(val),
            3 => {
                self.length.write_counter(val);
                self.timer.write_high(val);
                self.counter.reload = true;
            },
            _ => (),
        }
    }
}