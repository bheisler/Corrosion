//! Thin layer over BlipBuf which provides a slightly more convenient interface.

use blip_buf::BlipBuf;
use apu::Sample;
use std::cell::RefCell;
use std::rc::Rc;

const NES_CLOCK_RATE: u64 = 1789773;
const NES_FPS: usize = 60;
const FRAMES_PER_BUFFER: usize = 1;

pub struct SampleBuffer {
    blip: BlipBuf,
    samples: Vec<Sample>,
    transfer_samples: u32,
}

///Blip Buffer combined with a Vec to store the samples transferred out of the buffer, so we don't
///have to either allocate memory every transfer.
impl SampleBuffer {
    pub fn new(out_rate: f64) -> SampleBuffer {
        let samples_per_frame = out_rate as u32 / NES_FPS as u32;
        let transfer_samples = samples_per_frame * FRAMES_PER_BUFFER as u32;

        let mut buf = BlipBuf::new(transfer_samples);
        buf.set_rates(NES_CLOCK_RATE as f64, out_rate);
        let samples = vec![0; (transfer_samples) as usize];

        SampleBuffer {
            blip: buf,
            samples: samples,
            transfer_samples: transfer_samples,
        }
    }

    pub fn read(&mut self) -> &[Sample] {
        let samples_read = self.blip.read_samples(&mut self.samples, false);
        let slice: &[Sample] = &self.samples;
        &slice[0..samples_read]
    }

    pub fn add_delta(&mut self, clock_time: u32, delta: i32) {
        self.blip.add_delta(clock_time, delta)
    }

    pub fn end_frame(&mut self, clock_duration: u32) {
        self.blip.end_frame(clock_duration)
    }

    pub fn clocks_needed(&self) -> u32 {
        self.blip.clocks_needed(self.transfer_samples)
    }
}

///Allows multiple channels to share a SampleBuffer but maintain separate waveforms.
pub struct Waveform {
    buffer: Rc<RefCell<SampleBuffer>>,
    last_amp: Sample,
    volume_mult: i32,
}

impl Waveform {
    pub fn new(buffer: Rc<RefCell<SampleBuffer>>, volume_mult: i32) -> Waveform {
        Waveform {
            buffer: buffer,
            last_amp: 0,
            volume_mult: volume_mult,
        }
    }

    pub fn set_amplitude(&mut self, amp: Sample, cycle: u32) {
        let last_amp = self.last_amp;
        let delta = (amp - last_amp) as i32;
        if delta == 0 {
            return;
        }
        self.buffer.borrow_mut().add_delta(cycle, delta * self.volume_mult);
        self.last_amp = amp;
    }
}
