use super::memory::MemSegment;
use audio::AudioOut;
use std::cmp;
use blip_buf::BlipBuf;

pub type Sample = i16;

const NES_CLOCK_RATE: u64 = 1789773;
const CPU_CYCLES_PER_EVEN_TICK: u64 = 7438; //TODO: This is wrong.
const CPUCYCLES_PER_ODD_TICK: u64 = 7439;

const NES_FPS: usize = 60;
const FRAMES_PER_BUFFER : usize = 1;

const SAMPLE_RATE: usize = 44100;
const SAMPLES_PER_FRAME: usize = (SAMPLE_RATE / NES_FPS);
pub const BUFFER_SIZE: usize = SAMPLES_PER_FRAME * FRAMES_PER_BUFFER;

const VOLUME_MULT: i16 = (32767i16 / 16) / 2;

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

static PULSE_DUTY_CYCLES: [[i16; 8]; 4] = [
    [0, 1, -1, 0, 0, 0, 0, 0],
    [0, 1, 0, -1, 0, 0, 0, 0],
    [0, 1, 0, 0, 0, -1, 0, 0],
    [0, -1, 0, 1, 0, 0, 0, 0],
];

bitflags! {
    flags Frame : u8 {
        const MODE = 0b1000_0000, //0 = 4-step, 1 = 5-step
        const IRQ  = 0b0100_0000, //0 = disabled, 1 = enabled
    }
}

trait Writable {
    fn write(&mut self, idx: u16, val: u8);
}

#[derive(Debug)]
struct Length {
    halt_bit: usize,
    halted: bool,
    remaining: u8,
}

impl Length {
    fn write_halt(&mut self, val: u8) {
        self.halted = (val >> self.halt_bit ) & 0x01 != 0;
        if self.halted {
            self.remaining = 0;
        }
    }
    
    fn write_counter(&mut self, val: u8) {
        if !self.halted {
            self.remaining = LENGTH_TABLE[(val >> 3) as usize];
        }
    }
    
    fn tick(&mut self) {
        self.remaining = self.remaining.saturating_sub(1);
    }
    
    fn audible(&self) -> bool {
        self.remaining > 0
    }
    
    fn new(halt_bit: usize) -> Length {
        Length {
            halt_bit: halt_bit,
            halted: false,
            remaining: 0,
        }
    }
}

struct Envelope {
    should_loop: bool,
    disable: bool,
    n: u8,
    
    divider: u8,
    counter: u8,
}

impl Envelope {
    fn new() -> Envelope {
        Envelope {
            should_loop: false,
            disable: false,
            n: 0,
            divider: 0,
            counter: 0,
        }
    }
    
    fn write(&mut self, val: u8) {
        self.should_loop = (val >> 5) & 0x01 != 0;
        self.disable     = (val >> 6) & 0x01 != 0;
        self.n           = val & 0x0F;
        self.divider = self.n;
        self.counter = 15;
    }
    
    fn tick(&mut self) {
        if self.divider == 0 {
            self.envelope_tick();
            self.divider = self.n;
        }
        else {
            self.divider -= 1;
        }
    }
    
    fn envelope_tick(&mut self) {
        if self.should_loop && self.counter == 0 {
            self.counter = 15;
        }
        else {
            self.counter = self.counter.saturating_sub(1);
        }
    }
    
    fn volume(&self) -> i16 {
        if self.disable {
            self.n as i16
        }
        else {
            self.counter as i16
        }
    }
}

struct Timer {
    period: u16,
    current_step: u32,
    duty_index: usize,
    
    //The timer is clocked for every sample, so the period logic is in the Pulse.play function
}

impl Timer {
    fn new() -> Timer {
        Timer{
            period: 0,
            current_step: 0,
            duty_index: 0,
        }
    }
    
    fn write_low(&mut self, val: u8) {
        self.period = ( self.period & 0xFF00 ) | val as u16;
    } 
    
    fn write_high(&mut self, val: u8) {
        self.period = ( self.period & 0x00FF ) | (val as u16 & 0x0007) << 8;
    }
    
    fn add_period_shift(&mut self, shift: i16) {
        let new_period = (self.period as i16).wrapping_add( shift );
        self.period = new_period as u16;
    }
    
    fn wavelen(&self) -> u32 { (self.period as u32 + 1) * 2 }
}

struct Sweep {
    enable: bool,
    period: u8,
    negate: bool,
    shift: u8,
    
    is_pulse2: bool,
    divider: u8,
    reload: bool,
}

impl Sweep {
    fn new(is_pulse2: bool) -> Sweep {
        Sweep {
            enable: false,
            period: 0,
            negate: false,
            shift: 0,
            
            is_pulse2: is_pulse2,
            divider: 0,
            reload: false,
        }
    }
    
    fn write(&mut self, val: u8) {
        self.enable = (val & 0b1000_0000) != 0;
        self.period = (val & 0b0111_0000) >> 4;
        self.negate = (val & 0b0000_1000) != 0;
        self.shift  =  val & 0b0000_0111;
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
            timer.add_period_shift( period_shift );
        }
        
        if self.reload {
            self.divider = self.period;
            self.reload = false;
        }
    }
    
    fn audible(&self) -> bool {
        //TODO
        true
    }
    
    fn period_shift(&self, timer: &Timer) -> i16 {
        let mut shift = timer.period as i16;
        shift = shift >> self.shift;
        if self.negate {
            shift = -shift;
            if self.is_pulse2 {
                shift = shift + 1;
            }
        }
        shift
    }
}

struct Pulse {
    duty: usize,
    envelope: Envelope,
    sweep: Sweep,
    timer: Timer,
    length: Length,
    
    last_amp: Sample,
    buffer: SampleBuffer,
}

impl Pulse {
    fn new(is_pulse2: bool, buffer: SampleBuffer) -> Pulse {
        Pulse {
            duty: 0,
            envelope: Envelope::new(),
            sweep: Sweep::new(is_pulse2),
            timer: Timer::new(),
            length: Length::new(5),
            
            last_amp: 0,
            buffer: buffer,
        }
    }
    
    fn length_tick(&mut self) {
        self.length.tick();
        let timer = &mut self.timer;
        self.sweep.tick(timer)
    }
    
    fn envelope_tick(&mut self) {
        self.envelope.tick();
    }
    
    fn play(&mut self, from_cyc: u32, to_cyc: u32) {
        if !self.sweep.audible() ||
           !self.length.audible() {
           
           self.set_amplitude(0, from_cyc);
           return;
        }
        let volume = self.envelope.volume();
        
        let mut current_cyc = from_cyc;
        while current_cyc < to_cyc {
            let wavelen = self.timer.wavelen();
            let remaining = wavelen - self.timer.current_step;
            if remaining < to_cyc {
                self.timer.current_step = 0;
                current_cyc += remaining;
                self.timer.duty_index = ( self.timer.duty_index + 1 ) % 8;
                match PULSE_DUTY_CYCLES[self.duty][self.timer.duty_index] {
                    -1 => self.set_amplitude(0, current_cyc),
                    0 => (),
                    1 => self.set_amplitude(volume, current_cyc),
                    _ => (),
                };
            }
            else {
                self.timer.current_step = to_cyc - current_cyc;
                current_cyc = to_cyc;
            }
        }
    }
    
    fn set_amplitude(&mut self, amp: Sample, cycle: u32) {
        let last_amp = self.last_amp;
        let delta = (amp - last_amp) as i32;
        if delta == 0 {
            return;
        }
        self.buffer.add_delta(cycle, delta);
        self.last_amp = amp;
    }
}

impl Writable for Pulse {
    fn write(&mut self, idx: u16, val: u8) {
        match idx % 4 {
            0 => {
                self.duty = (val >> 6) as usize;
                self.length.write_halt(val);
                self.envelope.write(val);
            },
            1 => self.sweep.write(val),
            2 => self.timer.write_low(val),
            3 => { 
                self.length.write_counter(val);
                self.timer.write_high(val);
            },
            _ => (),
        }
    }
}

struct Triangle {
    counter: u8,
    timer: u8,
    length: Length,
}

impl Triangle {
    fn new() -> Triangle {
        Triangle {
            counter: 0,
            timer: 0,
            length: Length::new(7),
        }
    }
    
    fn length_tick(&mut self) {
        self.length.tick();
    }
    
    fn envelope_tick(&mut self) {
        //Nothing yet
    }
    
    fn play(&mut self, from_cyc: u32, to_cyc: u32) {
        
    }
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

struct Noise {
    envelope: Envelope,
    mode: u8,
    length: Length,
}

impl Noise {
    fn new() -> Noise {
        Noise {
            envelope: Envelope::new(),
            mode: 0,
            length: Length::new(5),
        }
    }
    
    fn length_tick(&mut self) {
        self.length.tick();
    }
    
    fn play(&mut self, from_cyc: u32, to_cyc: u32) {
        
    }
}

impl Writable for Noise {
    fn write(&mut self, idx: u16, val: u8) {
        match idx % 4 {
            0 => {
                self.length.write_halt(val);
                self.envelope.write(val);
            },
            1 => (),
            2 => (),
            3 => self.length.write_counter(val),
            _ => (),
        }
    }
}

struct DMC {
    freq: u8,
    direct: u8,
    sample_addr: u8,
    sample_length: u8,
}

impl DMC {
    fn new() -> DMC {
        DMC {
            freq: 0,
            direct: 0,
            sample_addr: 0,
            sample_length: 0,
        }
    }
    
    fn play(&mut self, from_cyc: u32, to_cyc: u32) {
        
    }
}

impl Writable for DMC {
    fn write(&mut self, idx: u16, val: u8) {
        
    }
}

struct SampleBuffer {
    blip: BlipBuf,
    samples: Vec<Sample>,
    transfer_samples: u32,
}

impl SampleBuffer {
    fn new(out_rate: f64) -> SampleBuffer {
        let samples_per_frame = out_rate as u32 / NES_FPS as u32;
        let transfer_samples = samples_per_frame * FRAMES_PER_BUFFER as u32;
        
        //TODO: This probably doesn't need to be so large.
        let mut buf = BlipBuf::new(transfer_samples * 2);
        buf.set_rates(NES_CLOCK_RATE as f64, out_rate);
        let samples = vec![0; (transfer_samples * 2) as usize];
        
        SampleBuffer {
            blip: buf,
            samples: samples,
            transfer_samples: transfer_samples,
        }
    }
    
    fn available(&self) -> u32 {
        self.blip.samples_avail()
    }
    
    fn read(&mut self) -> &[Sample] {
        let samples_read = self.blip.read_samples(&mut self.samples, false);
        let slice : &[Sample] = &self.samples;
        &slice[0..samples_read]
    }
    
    fn add_delta(&mut self, clock_time: u32, delta: i32) {
        self.blip.add_delta(clock_time, delta)
    }
    
    fn end_frame(&mut self, clock_duration: u32) {
        self.blip.end_frame(clock_duration)
    }
    
    fn clocks_needed(&self) -> u32 {
        self.blip.clocks_needed(self.transfer_samples)
    }
    
    fn clear(&mut self) {
        self.blip.clear()
    }
}

pub struct APU {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: DMC,
    frame: Frame,
    control: u8,
    status: u8,
    
    device: Box<AudioOut>,
    
    frame_count: usize,
    global_cyc: u64,
    tick: u64,
    next_tick_cyc: u64,
    next_transfer_cyc: u64,
    last_frame_cyc: u64,
}


impl APU {
    pub fn new( device: Box<AudioOut> ) -> APU {
        let sample_rate = device.sample_rate();
        let mut apu = APU {
            pulse1: Pulse::new(false, SampleBuffer::new(sample_rate)),
            pulse2: Pulse::new(true, SampleBuffer::new(sample_rate)),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: DMC::new(),
            frame: Frame::empty(),
            control: 0,
            status: 0,
            
            device: device,
            
            frame_count: 0,
            global_cyc: 0,
            tick: 0,
            next_tick_cyc: 0,
            next_transfer_cyc: 0,
            last_frame_cyc: 0,
        };
        apu.next_transfer_cyc = apu.pulse1.buffer.clocks_needed() as u64;
        apu
    }
    
    pub fn run_to(&mut self, cpu_cycle: u64) {
        while self.global_cyc < cpu_cycle {
            let current_cycle = self.global_cyc;
            
            let next_step = cmp::min(cmp::min(cpu_cycle, self.next_tick_cyc), self.next_transfer_cyc);
            self.play(current_cycle, next_step);
            self.global_cyc = next_step;
            
            if self.global_cyc == self.next_tick_cyc {
                self.tick();
            }
            if self.global_cyc == self.next_transfer_cyc {
                self.transfer();
            } 
        }
    }
    
    ///Represents the 240Hz output of the frame sequencer's divider
    fn tick(&mut self) {
        self.tick += 1;
        self.next_tick_cyc += if self.tick %2 == 0 {
            CPU_CYCLES_PER_EVEN_TICK
        }
        else {
            CPUCYCLES_PER_ODD_TICK
        };
                
        if !self.frame.contains(MODE) {
            match self.tick % 4 {
                0 => { self.envelope_tick(); },
                1 => { self.envelope_tick(); self.length_tick(); },
                2 => { self.envelope_tick(); },
                3 => { self.envelope_tick(); self.length_tick(); self.raise_irq(); },
                _ => (),
            }
        }
        else {
            match self.tick % 5 {
                0 => { self.envelope_tick(); self.length_tick() },
                1 => { self.envelope_tick(); },
                2 => { self.envelope_tick(); self.length_tick() },
                3 => { self.envelope_tick(); },
                4 => (),
                _ => (),
            }
        }
    }
    
    fn envelope_tick(&mut self) {
        self.pulse1.envelope_tick();
        self.pulse2.envelope_tick();
        self.triangle.envelope_tick();
    }
    
    fn length_tick(&mut self) {
        self.pulse1.length_tick();
        self.pulse2.length_tick();
        self.triangle.length_tick();
        self.noise.length_tick();
    }
    
    fn raise_irq(&mut self) {
        //TODO
    }
    
    fn play(&mut self, from_cyc: u64, to_cyc: u64) {
        let from = (from_cyc - self.last_frame_cyc) as u32;
        let to = (to_cyc - self.last_frame_cyc) as u32;
        self.pulse1.play(from, to);
        self.pulse2.play(from, to);
        self.triangle.play(from, to);
        self.noise.play(from, to);
        self.dmc.play(from, to);
    }
    
    fn transfer(&mut self) {
        let cpu_cyc = self.global_cyc;
        let cycles_since_last_frame = (cpu_cyc - self.last_frame_cyc) as u32;
        self.last_frame_cyc = cpu_cyc;
        self.pulse1.buffer.end_frame(cycles_since_last_frame);
        self.pulse2.buffer.end_frame(cycles_since_last_frame);
        let samples : Vec<Sample> = self.pulse1.buffer.read().iter().zip(self.pulse2.buffer.read().iter())
            .map(|(p1, p2)| p1 + p2)
            .map(|x| x * VOLUME_MULT)
            .collect();
        self.device.play(&samples);
        self.next_transfer_cyc = cpu_cyc + self.pulse1.buffer.clocks_needed() as u64;
    }
    
    pub fn frame_end(&mut self, cpu_cyc: u64) {
        self.run_to(cpu_cyc);
    }
}

impl MemSegment for APU {
    fn read(&mut self, idx: u16) -> u8 {
        match idx % 0x20 {
            0x0015 => 0, //TODO
            _ => 0,
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        match idx % 0x20 {
            x @ 0x00...0x03 => self.pulse1.write(x, val),
            x @ 0x04...0x07 => self.pulse2.write(x, val),
            x @ 0x08...0x0B => self.triangle.write(x, val),
            x @ 0x0C...0x0F => self.noise.write(x, val),
            x @ 0x10...0x13 => self.dmc.write(x, val),
            0x0014 => (),
            0x0015 => self.control = val,
            0x0016 => (),
            0x0017 => {
                self.frame = Frame::from_bits_truncate(val);
                self.tick = 0;
                if self.frame.contains( MODE ) {
                    //Trigger a tick immediately
                    self.next_tick_cyc = self.global_cyc
                }
                else {
                    //Reset the tick divider
                    self.next_tick_cyc = self.global_cyc + CPU_CYCLES_PER_EVEN_TICK
                }
            },
            _ => (),
        }
    }
}
