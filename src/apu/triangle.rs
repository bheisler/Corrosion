//! Contains structures used by the NES's triangle channel.

use apu::Writable;
use apu::buffer::Waveform;
use apu::components::*;

#[cfg_attr(rustfmt, rustfmt_skip)]
static TRIANGLE_VOLUME: [i16; 32] = [
    0xF, 0xE, 0xD, 0xC, 0xB, 0xA, 0x9, 0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1, 0x0,
    0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF];

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
        } else {
            self.counter = self.counter.saturating_sub(1);
        }

        if !self.control {
            self.reload = false;
        }
    }

    fn audible(&self) -> bool {
        self.counter > 0
    }
}

pub struct Triangle {
    counter: LinearCounter,
    timer: Timer,
    pub length: Length,

    waveform: Waveform,
    volume_index: usize,
}

impl Triangle {
    pub fn new(waveform: Waveform) -> Triangle {
        Triangle {
            counter: LinearCounter::new(),
            timer: Timer::new(1),
            length: Length::new(7),

            waveform: waveform,
            volume_index: 0,
        }
    }

    pub fn length_tick(&mut self) {
        self.length.tick();
    }

    pub fn envelope_tick(&mut self) {
        self.counter.tick();
    }

    pub fn play(&mut self, from_cyc: u32, to_cyc: u32) {
        if !self.counter.audible() || !self.length.audible() {
            self.waveform.set_amplitude(0, from_cyc);
            return;
        }

        let mut current_cycle = from_cyc;
        while let TimerClock::Clock = self.timer.run(&mut current_cycle, to_cyc) {
            self.volume_index = (self.volume_index + 1) % 32;
            let volume = TRIANGLE_VOLUME[self.volume_index];
            self.waveform.set_amplitude(volume, current_cycle);
        }
    }
}

impl Writable for Triangle {
    fn write(&mut self, idx: u16, val: u8) {
        match idx % 4 {
            0 => {
                self.length.write_halt(val);
                self.counter.write(val);
            }
            1 => (),
            2 => self.timer.write_low(val),
            3 => {
                self.length.write_counter(val);
                self.timer.write_high(val);
                self.counter.reload = true;
            }
            _ => (),
        }
    }
}
