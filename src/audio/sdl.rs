use super::AudioOut;
use ::apu::OutputBuffer;
use sdl2::Sdl;
use sdl2::AudioSubsystem;
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};
use std::sync::{Mutex, Condvar};
use std::sync::Arc;

const OUT_SAMPLE_RATE: i32 = 44100;
const BUFFER_SIZE : usize = OUT_SAMPLE_RATE as usize / ::apu::BUFFERS_PER_SECOND;

struct BufferOut {
    samples: [f32; BUFFER_SIZE],
    playback_counter: usize,
    condvar: Arc<Condvar>,
}

impl AudioCallback for BufferOut {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let out_len = out.len();
        let output_iter = out.iter_mut();
        let playback_counter = self.playback_counter;
        let input_iter = self.samples.iter().skip(playback_counter).take(out_len);
        
        let mut transferred : usize = 0;
        for (dest, src) in output_iter.zip( input_iter ) { 
            *dest = *src;
            transferred += 1;
        }
        
        self.playback_counter += transferred;
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
    fn play(&mut self, buffer: &OutputBuffer) {
        self.wait();
        let mut out = self.device.lock();
        out.playback_counter = 0;
        let dest_iter = out.samples.iter_mut();
        let src_iter = buffer.samples.iter();
        
        for (dest, src) in dest_iter.zip( src_iter ) {
            *dest = *src;
        }
    }
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
            samples: Some(BUFFER_SIZE as u16),
        };
    
        let device = audio_subsystem.open_playback(None, desired_spec, |_| {
            BufferOut {
                samples: [0f32; BUFFER_SIZE],
                playback_counter: 0,
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
    
    fn wait(&mut self) {
        loop {
            let lock = self.mutex.lock().unwrap();
            let _lock = self.condvar.wait(lock).unwrap();
            let callback = self.device.lock();
            if callback.playback_counter == callback.samples.len() {
                return;
            }
        }
    }
}