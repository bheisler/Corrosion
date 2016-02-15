use super::AudioOut;
use ::apu::Sample;
use sdl2::Sdl;
use sdl2::AudioSubsystem;
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};
use std::sync::{Mutex, Condvar};
use std::sync::Arc;
use std::cmp;

const OUT_SAMPLE_RATE: i32 = 44100;
const BUFFER_SIZE : usize = OUT_SAMPLE_RATE as usize;

struct BufferOut {
    samples: [Sample; BUFFER_SIZE],
    input_counter: usize,
    playback_counter: usize,
    input_samples: usize,
    too_slow: bool,
    condvar: Arc<Condvar>,
}

impl AudioCallback for BufferOut {
    type Channel = Sample;

    fn callback(&mut self, out: &mut [Sample]) {
        {
            let out_iter = out.iter_mut();
            let in_iter = self.samples.iter()
                .cycle()
                .skip(self.playback_counter)
                .take(self.input_samples); 
            
            for (dest, src) in out_iter.zip(in_iter) {
                *dest = *src;
            }
        }
        
        let transferred = cmp::min(out.len(), self.input_samples);
        self.input_samples = self.input_samples - transferred;
        self.playback_counter = (self.playback_counter + transferred) % self.samples.len();

        {
            let out_iter = out.iter_mut().skip(transferred);
            //This should rarely, if ever, execute.
            for dest in out_iter {
                self.too_slow = true;
                *dest = 0;
            }
        }
        
        self.condvar.notify_one();
    }
}

#[allow(dead_code)]
pub struct SDLAudioOut {
    system: AudioSubsystem,
    device: AudioDevice<BufferOut>,
    mutex: Mutex<()>,
    condvar: Arc<Condvar>,
}

impl AudioOut for SDLAudioOut {
    fn play(&mut self, buffer: &[Sample]) {
        self.wait(buffer.len());
        let mut out = self.device.lock();
        
        if out.too_slow {
            println!("Audio transfer can't keep up");
            out.too_slow = false;
        }
        
        let mut in_index = 0;
        let mut out_index = out.input_counter;
        let out_len = out.samples.len();
        let in_len = buffer.len();
        
        while in_index < in_len {
            out.samples[out_index] = buffer[in_index];
            in_index += 1;
            out_index += 1;
            if out_index == out_len {
                out_index = 0;
            }
        }
        out.input_counter = (out.input_counter + in_len) % out_len;
        out.input_samples = out.input_samples + in_len;
    }
    
    fn sample_rate(&self) -> f64 { OUT_SAMPLE_RATE as f64 }
}

impl SDLAudioOut {
    pub fn new( sdl: &Sdl ) -> SDLAudioOut {
        let mutex = Mutex::new(());
        let condvar = Condvar::new();
        let condvar = Arc::new(condvar);
        
        let audio_subsystem = sdl.audio().unwrap();
    
        let desired_spec = AudioSpecDesired {
            freq: Some(OUT_SAMPLE_RATE),
            channels: Some(1),
            samples: None,
        };
    
        let device = audio_subsystem.open_playback(None, desired_spec, |_| {
            BufferOut {
                samples: [0; BUFFER_SIZE],
                input_counter: 0,
                playback_counter: 0,
                input_samples: 0,
                too_slow: false,
                condvar: condvar.clone(),
            }
        }).unwrap();
    
        // Start playback
        device.resume();
        
        SDLAudioOut {
            system: audio_subsystem,
            device: device,
            mutex: mutex,
            condvar: condvar,
        }
    }
    
    fn wait(&mut self, in_size: usize) {
        {
            let callback = self.device.lock();
            if callback.input_samples + in_size <= callback.samples.len() {
                return;
            }
        }
        
        //If there isn't enough room for the transfer, wait until the callback is called once,
        //then check again.
        loop {
            let lock = self.mutex.lock().unwrap();
            let _lock = self.condvar.wait(lock).unwrap();
            let callback = self.device.lock();
            if callback.input_samples + in_size <= callback.samples.len() {
                return;
            }
        }
    }
}