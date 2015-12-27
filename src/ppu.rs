use super::memory::MemSegment;

pub struct PPU {
    ppuctrl: u8,
    ppumask: u8,
    ppustat: u8,
    oamaddr: u8,
    ppuscroll: u16,
    ppuaddr: u16,

    oam: [u8; 256],

    ///A fake dynamic latch representing the capacitance of the wires in the
    ///PPU that we have to emulate.
    dyn_latch: u8,

    ///Whether we're writing into the first (false) or second (true) byte of the
    ///address registers.
    address_latch: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            ppuctrl: 0,
            ppumask: 0,
            ppustat: 0,
            oamaddr: 0,
            ppuscroll: 0,
            ppuaddr: 0,
            dyn_latch: 0,
            address_latch: false,
            oam: [0u8; 256],
        }
    }
}

impl MemSegment for PPU {
    fn read(&mut self, idx: u16) -> u8 {
        match idx % 8 {
            0x0000 => self.dyn_latch,
            0x0001 => self.dyn_latch,
            0x0002 => {
                self.address_latch = false;
                (self.ppustat & 0b1110_0000) | (self.dyn_latch & 0b0001_1111)
            }
            0x0003 => self.dyn_latch,
            0x0004 => {
                let res = self.oam[self.oamaddr as usize];
                self.oamaddr = self.oamaddr.wrapping_add(1);
                res
            }
            0x0005 => self.dyn_latch,
            0x0006 => self.dyn_latch,
            0x0007 => 0u8,
            x => invalid_address!(x),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        self.dyn_latch = val;
        match idx % 8 {
            0x0000 => self.ppuctrl = val,
            0x0001 => self.ppumask = val,
            0x0002 => (),
            0x0003 => self.oamaddr = val,
            0x0004 => {
                self.oam[self.oamaddr as usize] = val;
                self.oamaddr = self.oamaddr.wrapping_add(1);
            }
            0x0005 => {
                if self.address_latch {
                    self.ppuscroll = (self.ppuscroll & 0xFF00) | ((val as u16) << 0);
                } else {
                    self.ppuscroll = (self.ppuscroll & 0x00FF) | ((val as u16) << 8);
                }
                self.address_latch = true;
            }
            0x0006 => {
                if self.address_latch {
                    self.ppuaddr = (self.ppuaddr & 0xFF00) | ((val as u16) << 0);
                } else {
                    self.ppuaddr = (self.ppuaddr & 0x00FF) | ((val as u16) << 8);
                }
                self.address_latch = true;
            }
            0x0007 => (),
            x => invalid_address!(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory::MemSegment;

    fn assert_register_single_writable(idx: u16, getter: &Fn(&PPU) -> u8) {
        let mut ppu = PPU::new();
        ppu.write(idx, 12);
        assert_eq!(getter(&ppu), 12);
        ppu.write(idx, 125);
        assert_eq!(getter(&ppu), 125);
    }

    fn assert_register_double_writable(idx: u16, getter: &Fn(&PPU) -> u16) {
        let mut ppu = PPU::new();
        ppu.write(idx, 0xDE);
        assert_eq!(getter(&ppu), 0xDE00);
        ppu.write(idx, 0xAD);
        assert_eq!(getter(&ppu), 0xDEAD);
        ppu.write(idx, 0xED);
        assert_eq!(getter(&ppu), 0xDEED);
        ppu.address_latch = false;
        ppu.write(idx, 0xAD);
        assert_eq!(getter(&ppu), 0xADED);
    }

    fn assert_register_ignores_writes(idx: u16, getter: &Fn(&PPU) -> u8) {
        let mut ppu = PPU::new();
        ppu.write(idx, 12);
        assert_eq!(getter(&ppu), 0);
        ppu.write(idx, 125);
        assert_eq!(getter(&ppu), 0);
    }

    fn assert_writing_register_fills_latch(idx: u16) {
        let mut ppu = PPU::new();
        ppu.write(idx, 12);
        assert_eq!(ppu.dyn_latch, 12);
        ppu.write(idx, 125);
        assert_eq!(ppu.dyn_latch, 125);
    }

    fn assert_register_is_readable(idx: u16, setter: &Fn(&mut PPU, u8) -> ()) {
        let mut ppu = PPU::new();
        setter(&mut ppu, 12);
        assert_eq!(ppu.read(idx), 12);
        setter(&mut ppu, 125);
        assert_eq!(ppu.read(idx), 125);
    }

    fn assert_register_not_readable(idx: u16) {
        let mut ppu = PPU::new();
        ppu.dyn_latch = 12;
        assert_eq!(ppu.read(idx), 12);
        ppu.dyn_latch = 125;
        assert_eq!(ppu.read(idx), 125);
    }

    #[test]
    fn ppuctrl_is_write_only_register() {
        assert_register_single_writable(0x2000, &|ref ppu| ppu.ppuctrl);
        assert_writing_register_fills_latch(0x2000);
        assert_register_not_readable(0x2000);
    }

    #[test]
    fn ppu_mirrors_address() {
        assert_register_single_writable(0x2008, &|ref ppu| ppu.ppuctrl);
        assert_register_single_writable(0x2010, &|ref ppu| ppu.ppuctrl);
    }

    #[test]
    fn ppumask_is_write_only_register() {
        assert_register_single_writable(0x2001, &|ref ppu| ppu.ppumask);
        assert_writing_register_fills_latch(0x2001);
        assert_register_not_readable(0x2001);
    }

    #[test]
    fn ppustat_is_read_only_register() {
        assert_register_ignores_writes(0x2002, &|ref ppu| ppu.ppustat);
        assert_writing_register_fills_latch(0x2002);
        assert_register_is_readable(0x2002,
                                    &|ref mut ppu, val| {
                                        ppu.ppustat = val;
                                        ppu.dyn_latch = val;
                                    });
    }

    #[test]
    fn reading_ppustat_returns_part_of_dynlatch() {
        let mut ppu = PPU::new();
        ppu.dyn_latch = 0b0001_0101;
        ppu.ppustat = 0b1010_0101;
        assert_eq!(ppu.read(0x2002), 0b1011_0101);
    }

    #[test]
    fn reading_ppustat_clears_addr_latch() {
        let mut ppu = PPU::new();
        ppu.address_latch = true;
        ppu.read(0x2002);
        assert_eq!(ppu.address_latch, false);
    }

    #[test]
    fn oamaddr_is_write_only_register() {
        assert_register_single_writable(0x2003, &|ref ppu| ppu.oamaddr);
        assert_writing_register_fills_latch(0x2003);
        assert_register_not_readable(0x2003);
    }

    #[test]
    fn ppuscroll_is_2x_write_only_register() {
        assert_register_double_writable(0x2005, &|ref ppu| ppu.ppuscroll);
        assert_writing_register_fills_latch(0x2005);
        assert_register_not_readable(0x2005);
    }

    #[test]
    fn ppuaddr_is_2x_write_only_register() {
        assert_register_double_writable(0x2006, &|ref ppu| ppu.ppuaddr);
        assert_writing_register_fills_latch(0x2006);
        assert_register_not_readable(0x2006);
    }

    #[test]
    fn reading_oamdata_uses_oamaddr_as_index_into_oam() {
        let mut ppu = PPU::new();
        for x in 0..255 {
            ppu.oam[x] = (255 - x) as u8;
        }
        ppu.oamaddr = 0;
        assert_eq!(ppu.read(0x2004), 255);
        ppu.oamaddr = 10;
        assert_eq!(ppu.read(0x2004), 245);
    }

    #[test]
    fn reading_oamdata_increments_oamaddr() {
        let mut ppu = PPU::new();
        ppu.oamaddr = 0;
        ppu.read(0x2004);
        assert_eq!(ppu.oamaddr, 1);
        ppu.oamaddr = 255;
        ppu.read(0x2004);
        assert_eq!(ppu.oamaddr, 0);
    }

    #[test]
    fn writing_oamdata_uses_oamaddr_as_index_into_oam() {
        let mut ppu = PPU::new();
        ppu.oamaddr = 0;
        ppu.write(0x2004, 12);
        assert_eq!(ppu.oam[0], 12);
        ppu.oamaddr = 10;
        ppu.write(0x2004, 15);
        assert_eq!(ppu.oam[10], 15);
    }

    #[test]
    fn writing_oamdata_increments_oamaddr() {
        let mut ppu = PPU::new();
        ppu.oamaddr = 0;
        ppu.write(0x2004, 12);
        assert_eq!(ppu.oamaddr, 1);
        ppu.oamaddr = 255;
        ppu.write(0x2004, 12);
        assert_eq!(ppu.oamaddr, 0);
    }
}
