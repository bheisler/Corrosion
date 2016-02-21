//! Contains structures used only by the NES's two square-wave channels.

use apu::Writable;
use apu::components::*;
use apu::buffer::*;

static SQUARE_DUTY_CYCLES: [[i16; 8]; 4] = [[0, 1, -1, 0, 0, 0, 0, 0],
                                           [0, 1, 0, -1, 0, 0, 0, 0],
                                           [0, 1, 0, 0, 0, -1, 0, 0],
                                           [0, -1, 0, 1, 0, 0, 0, 0]];

///Represents the frequency-sweep units used by the two square channels.
struct Sweep {
    enable: bool,
    period: u8,
    negate: bool,
    shift: u8,

    is_square2: bool,
    divider: u8,
    reload: bool,
}

impl Sweep {
    fn new(is_square2: bool) -> Sweep {
        Sweep {
            enable: false,
            period: 0,
            negate: false,
            shift: 0,

            is_square2: is_square2,
            divider: 0,
            reload: false,
        }
    }

    fn write(&mut self, val: u8) {
        self.enable = (val & 0b1000_0000) != 0;
        self.period = (val & 0b0111_0000) >> 4;
        self.negate = (val & 0b0000_1000) != 0;
        self.shift = val & 0b0000_0111;
        self.reload = true;
    }

    fn tick(&mut self, timer: &mut Timer) {
        if !self.enable {
            return;
        }

        self.divider = self.divider.saturating_sub(1);
        if self.divider == 0 {
            self.divider = self.period;
            let period_shift = self.period_shift(timer);
            timer.add_period_shift(period_shift);
        }

        if self.reload {
            self.divider = self.period;
            self.reload = false;
        }
    }

    fn audible(&self) -> bool {
        // TODO
        true
    }

    fn period_shift(&self, timer: &Timer) -> i16 {
        let mut shift = timer.period() as i16;
        shift = shift >> self.shift;
        if self.negate {
            shift = -shift;
            if self.is_square2 {
                shift = shift + 1;
            }
        }
        shift
    }
}
pub struct Square {
    duty: usize,
    duty_index: usize,
    
    envelope: Envelope,
    sweep: Sweep,
    timer: Timer,
    pub length: Length,

	waveform: Waveform,
}

impl Square {
    pub fn new(is_square2: bool, waveform: Waveform) -> Square {
        Square {
            duty: 0,
            duty_index: 0,
            
            envelope: Envelope::new(),
            sweep: Sweep::new(is_square2),
            timer: Timer::new(2),
            length: Length::new(5),

            waveform: waveform,
        }
    }

    pub fn length_tick(&mut self) {
        self.length.tick();
        let timer = &mut self.timer;
        self.sweep.tick(timer)
    }

    pub fn envelope_tick(&mut self) {
        self.envelope.tick();
    }

    pub fn play(&mut self, from_cyc: u32, to_cyc: u32) {
        if !self.sweep.audible() || !self.length.audible() {
            self.waveform.set_amplitude(0, from_cyc);
            return;
        }

        let volume = self.envelope.volume();

        let mut current_cyc = from_cyc;
        while let TimerClock::Clock = self.timer.run(&mut current_cyc, to_cyc) {
            self.duty_index = (self.duty_index + 1) % 8;
            match SQUARE_DUTY_CYCLES[self.duty][self.duty_index] {
                -1 => self.waveform.set_amplitude(0, current_cyc),
                0 => (),
                1 => self.waveform.set_amplitude(volume, current_cyc),
                _ => (),
            };
        }
    }
}

impl Writable for Square {
    fn write(&mut self, idx: u16, val: u8) {
        match idx % 4 {
            0 => {
                self.duty = (val >> 6) as usize;
                self.length.write_halt(val);
                self.envelope.write(val);
            }
            1 => self.sweep.write(val),
            2 => self.timer.write_low(val),
            3 => {
                self.length.write_counter(val);
                self.timer.write_high(val);
            }
            _ => (),
        }
    }
}