#![macro_use]

macro_rules! invalid_address {
    ($e:expr) => (panic!("Invalid NES Memory Address: {:X}", $e));
}

use std::ops::Range;

pub trait MemSegment {
    fn read(&mut self, idx: u16) -> u8;
    fn read_w(&mut self, idx: u16) -> u16 {
        let low = self.read(idx) as u16;
        let high = self.read(idx + 1) as u16;
        (high << 8) | low
    }

    fn write(&mut self, idx: u16, val: u8);
    fn write_w(&mut self, idx: u16, val: u16) {
        let low = (val & 0x00FF) as u8;
        let high = ((val & 0xFF00) >> 8) as u8;
        self.write(idx, low);
        self.write(idx + 1, high);
    }

    fn print(&mut self, range: Range<u16>) {
        self.print_columns(range, 16)
    }

    fn print_columns(&mut self, range: Range<u16>, columns: u16) {
        let lower = range.start / columns;
        let upper = (range.end + columns - 1) / columns;

        for y in lower..upper {
            print!("{:04X}: ", y * columns);
            for x in 0..columns {
                let addr = (y * columns) + x;
                print!("{:02X} ", self.read(addr));
                if x % 4 == 3 {
                    print!(" ");
                }
            }
            println!("");
        }
    }
}
