//! This module contains implementations of the common components used by the
//! various NES sound channels.

#[cfg_attr(rustfmt, rustfmt_skip)]
static LENGTH_TABLE: [u8; 32] = [
    0x0A, 0xFE,
    0x14, 0x02,
    0x28, 0x04,
    0x50, 0x06,
    0xA0, 0x08,
    0x3C, 0x0A,
    0x0E, 0x0C,
    0x1A, 0x0E,
    0x0C, 0x10,
    0x18, 0x12,
    0x30, 0x14,
    0x60, 0x16,
    0xC0, 0x18,
    0x48, 0x1A,
    0x10, 0x1C,
    0x20, 0x1E,
];

///Represents the Length counter used by all NES sound channels except the DMC.
#[derive(Debug)]
pub struct Length {
    halt_bit: usize,
    halted: bool,
    enabled: bool,
    remaining: u8,
}

impl Length {
    pub fn write_halt(&mut self, val: u8) {
        self.halted = (val >> self.halt_bit) & 0x01 != 0;
    }

    pub fn write_counter(&mut self, val: u8) {
        if self.enabled {
            self.remaining = LENGTH_TABLE[(val >> 3) as usize];
        }
    }

    pub fn tick(&mut self) {
        if !self.halted {
            self.remaining = self.remaining.saturating_sub(1);
        }
    }

    pub fn audible(&self) -> bool {
        self.remaining > 0
    }

    pub fn active(&self) -> u8 {
        if self.audible() {
            1
        } else {
            0
        }
    }

    pub fn set_enable(&mut self, enable: bool) {
        self.enabled = enable;
        if !enable {
            self.remaining = 0;
        }
    }

    pub fn new(halt_bit: usize) -> Length {
        Length {
            halt_bit: halt_bit,
            halted: false,
            enabled: false,
            remaining: 0,
        }
    }
}

///Represents the Envelope Generator (volume setting) used by the pulse & noise channels.
#[derive(Debug)]
pub struct Envelope {
    should_loop: bool,
    disable: bool,
    n: u8,

    divider: u8,
    counter: u8,
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            should_loop: false,
            disable: false,
            n: 0,
            divider: 0,
            counter: 0,
        }
    }

    pub fn write(&mut self, val: u8) {
        self.should_loop = (val >> 5) & 0x01 != 0;
        self.disable = (val >> 6) & 0x01 != 0;
        self.n = val & 0x0F;
        self.divider = self.n;
        self.counter = 15;
    }

    pub fn tick(&mut self) {
        if self.divider == 0 {
            self.envelope_tick();
            self.divider = self.n;
        } else {
            self.divider -= 1;
        }
    }

    pub fn envelope_tick(&mut self) {
        if self.should_loop && self.counter == 0 {
            self.counter = 15;
        } else {
            self.counter = self.counter.saturating_sub(1);
        }
    }

    pub fn volume(&self) -> i16 {
        if self.disable {
            self.n as i16
        } else {
            self.counter as i16
        }
    }
}

#[derive(Debug)]
pub enum TimerClock {
    Clock,
    NoClock,
}

///Represents the CPU-clock timers used by all of the NES channels.
#[derive(Debug)]
pub struct Timer {
    period: u16,
    divider: u32,
    remaining: u32,
}

impl Timer {
    pub fn new(divider: u32) -> Timer {
        Timer {
            period: 0,
            divider: divider,
            remaining: 0,
        }
    }

    pub fn write_low(&mut self, val: u8) {
        self.period = (self.period & 0xFF00) | val as u16;
    }

    pub fn write_high(&mut self, val: u8) {
        self.period = (self.period & 0x00FF) | (val as u16 & 0x0007) << 8;
    }

    pub fn add_period_shift(&mut self, shift: i16) {
        let new_period = (self.period as i16).wrapping_add(shift);
        self.period = new_period as u16;
    }

    pub fn period(&self) -> u16 {
        self.period
    }

    fn wavelen(&self) -> u32 {
        (self.period as u32 + 1) * self.divider
    }

    ///Run the timer until the next clock, or until current_cyc reaches to_cycle.
    ///Returns either Clock or NoClock depending on if it reached a clock or not.
    pub fn run(&mut self, current_cyc: &mut u32, to_cyc: u32) -> TimerClock {
        let end_wavelen = *current_cyc + self.remaining;

        if end_wavelen < to_cyc {
            *current_cyc += self.remaining;
            self.remaining = self.wavelen();
            TimerClock::Clock
        } else {
            self.remaining -= to_cyc - *current_cyc;
            *current_cyc = to_cyc;
            TimerClock::NoClock
        }
    }
}
