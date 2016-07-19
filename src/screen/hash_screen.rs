extern crate sha1;

use ppu::{Color, SCREEN_BUFFER_SIZE};
use screen::Screen;
use self::sha1::{Digest, Sha1};
use std::collections::HashMap;

fn hash_screen(buf: &[Color; SCREEN_BUFFER_SIZE]) -> Digest {
    let newbuf: Vec<u8> = buf.iter()
        .map(|col: &Color| col.bits())
        .collect();

    let mut s = Sha1::new();
    s.update(&newbuf);
    s.digest()
}

pub struct HashPrinter {
    frames: u32,

    delegate: Box<Screen>,
}

impl HashPrinter {
    fn new(delegate: Box<Screen>) -> HashPrinter {
        HashPrinter {
            frames: 0,
            delegate: delegate,
        }
    }
}

impl Screen for HashPrinter {
    fn draw(&mut self, buf: &[Color; SCREEN_BUFFER_SIZE]) {
        println!("Frame: {}, Hash: {}",
                 self.frames,
                 hash_screen(buf).to_string());
        self.frames += 1;

        self.delegate.draw(buf);
    }
}

pub struct HashVerifier {
    hashes: HashMap<u32, &'static str>,
    frames: u32,
}

impl HashVerifier {
    fn new(hashes: HashMap<u32, &'static str>) -> HashVerifier {
        HashVerifier {
            frames: 0,
            hashes: hashes,
        }
    }
}

impl Screen for HashVerifier {
    fn draw(&mut self, buf: &[Color; SCREEN_BUFFER_SIZE]) {
        if self.hashes.contains_key(&self.frames) {
            assert_eq!(self.hashes.get(&self.frames).unwrap(),
                       &hash_screen(buf).to_string());
        }
        self.frames += 1;
    }
}
