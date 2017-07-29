use memmap::{Mmap, Protection};
use memory::MemSegment;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;

pub struct BatteryBackedRam {
    file: Mmap,
    len: u32,
}

impl BatteryBackedRam {
    pub fn new(rom_path: &Path, size: u32) -> io::Result<BatteryBackedRam> {
        let sav_path = rom_path.to_path_buf().with_extension("sav");
        let file = try!(
            OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(sav_path)
        );
        try!(file.set_len(size as u64));
        let file = try!(Mmap::open(&file, Protection::ReadWrite));
        Ok(BatteryBackedRam {
            file: file,
            len: size,
        })
    }

    fn wrap_addr(&self, idx: u16) -> usize {
        let idx = idx as usize;
        idx % self.len as usize
    }

    fn slice(&mut self) -> &mut [u8] {
        unsafe { self.file.as_mut_slice() }
    }
}

impl MemSegment for BatteryBackedRam {
    fn read(&mut self, idx: u16) -> u8 {
        let addr = self.wrap_addr(idx);
        self.slice()[addr]
    }

    fn write(&mut self, idx: u16, val: u8) {
        let addr = self.wrap_addr(idx);
        self.slice()[addr] = val;
        self.file.flush_async().unwrap();
    }
}
