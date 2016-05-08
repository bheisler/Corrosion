//! Contains structures used by the NES's noise channel.

use apu::Writable;
use apu::components::*;
use apu::buffer::*;

static PERIOD_TABLE: [u16; 16] = [0x0004, 0x0008, 0x0010, 0x0020, 0x0040, 0x0060, 0x0080, 0x00A0,
                                  0x00CA, 0x00FE, 0x017C, 0x01FC, 0x02FA, 0x03F8, 0x07F2, 0x0FE4];

struct LinearFeedbackShiftRegister {
    value: u16,
    mode: u8,
}

impl LinearFeedbackShiftRegister {
    fn new() -> LinearFeedbackShiftRegister {
        LinearFeedbackShiftRegister {
            value: 1,
            mode: 0,
        }
    }

    fn shift(&mut self) -> bool {
        let bit0 = self.value & 0x01;
        let bit1 = self.other_bit();

        let new_bit = bit0 ^ bit1;

        self.value = (self.value >> 1) | (new_bit << 14);
        self.value & 0x01 == 1
    }

    fn other_bit(&self) -> u16 {
        if self.mode == 0 {
            (self.value & (0x01 << 1)) >> 1
        } else {
            (self.value & (0x01 << 6)) >> 6
        }
    }

    fn set_mode(&mut self, mode: u8) {
        self.mode = mode;
    }
}

pub struct Noise {
    envelope: Envelope,
    pub length: Length,

    timer: Timer,
    shifter: LinearFeedbackShiftRegister,

    waveform: Waveform,
}

impl Noise {
    pub fn new(waveform: Waveform) -> Noise {
        Noise {
            envelope: Envelope::new(),
            length: Length::new(5),

            timer: Timer::new(1),

            waveform: waveform,

            shifter: LinearFeedbackShiftRegister::new(),
        }
    }

    pub fn length_tick(&mut self) {
        self.length.tick();
    }

    pub fn envelope_tick(&mut self) {
        self.envelope.tick();
    }

    pub fn play(&mut self, from_cyc: u32, to_cyc: u32) {
        if !self.length.audible() {
            self.waveform.set_amplitude(0, from_cyc);
            return;
        }

        let volume = self.envelope.volume();

        let mut current_cyc = from_cyc;
        while let TimerClock::Clock = self.timer.run(&mut current_cyc, to_cyc) {
            let enabled = self.shifter.shift();
            let amp = if enabled {
                volume
            } else {
                0
            };
            self.waveform.set_amplitude(amp, current_cyc);
        }
    }
}

impl Writable for Noise {
    fn write(&mut self, idx: u16, val: u8) {
        match idx % 4 {
            0 => {
                self.length.write_halt(val);
                self.envelope.write(val);
            }
            2 => {
                let mode = (val & 0b1000_0000) >> 7;
                self.shifter.set_mode(mode);
                let period_index = val & 0b0000_1111;
                self.timer.set_period(PERIOD_TABLE[period_index as usize]);
            }
            3 => self.length.write_counter(val),
            _ => (),
        }
    }
}
