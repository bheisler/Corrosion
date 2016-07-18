extern crate sha1;

use ppu::{Color, SCREEN_BUFFER_SIZE};
use screen::Screen;
use sha1::Sha1;

pub struct HashScreen {
    frames: u64,
}

impl Default for HashScreen {
    fn default() -> HashScreen {
        HashScreen { frames: 0 }
    }
}

impl Screen for HashScreen {
    fn draw(&mut self, buf: &[Color; SCREEN_BUFFER_SIZE]) {
        let newbuf: Vec<u8> = buf.iter()
            .map(|col: &Color| col.bits())
            .collect();

        let mut s = Sha1::new();
        s.update(&newbuf);
        println!("Frame: {}, Hash: {}", self.frames, s.digest().to_string());
        self.frames += 1;
    }
}
