use memory::MemSegment;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AddrByte {
    High,
    Low,
}

pub struct PPUCtrl {
    bits: u8,
}
impl PPUCtrl {
    pub fn empty() -> PPUCtrl {
        PPUCtrl { bits: 0 }
    }

    pub fn new(bits: u8) -> PPUCtrl {
        PPUCtrl { bits: bits }
    }

    pub fn nametable_num(&self) -> u16 {
        (self.bits & 0b0000_0011) as u16
    }

    pub fn vram_addr_step(&self) -> u16 {
        if self.bits & 0b0000_0100 != 0 {
            32
        } else {
            1
        }
    }

    pub fn background_table(&self) -> u16 {
        if self.bits & 0b0001_0000 != 0 {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn sprite_table(&self) -> u16 {
        if self.bits & 0b0010_0000 != 0 {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn generate_vblank_nmi(&self) -> bool {
        self.bits & 0b1000_0000 != 0
    }
}

bitflags! {
    flags PPUMask : u8 {
        const GREY =    0b0000_0001, //Greyscale
        const S_BCK_L = 0b0000_0010, //Show background in the leftmost 8 pixels
        const S_SPR_L = 0b0000_0100, //Show sprites in the leftmost 8 pixels
        const S_BCK =   0b0000_1000, //Show background
        const S_SPR =   0b0001_0000, //Show sprites
        const EM_R =    0b0010_0000, //Emphasize Red
        const EM_G =    0b0100_0000, //Emphasize Green
        const EM_B =    0b1000_0000, //Emphasize Blue
    }
}

bitflags! {
    flags PPUStat : u8 {
        const VBLANK =          0b1000_0000, //Currently in the vertical blank interval
        const SPRITE_0 =        0b0100_0000, //Sprite 0 hit
        const SPRITE_OVERFLOW = 0b0010_0000, //Greater than 8 sprites on current scanline
    }
}

pub struct PPUReg {
    pub ppuctrl: PPUCtrl,
    pub ppumask: PPUMask,
    pub ppustat: PPUStat,
    pub oamaddr: u8,
    ppuscroll: u16,
    pub ppuaddr: u16,

    ///A fake dynamic latch representing the capacitance of the wires in the
    ///PPU that we have to emulate.
    dyn_latch: u8,

    ///The address registers are two bytes but we can only write one at a time.
    address_latch: AddrByte,
}

impl PPUReg {
    pub fn scroll_x(&self) -> u16 {
        (self.ppuscroll & 0xFF00) >> 8
    }

    pub fn scroll_x_fine(&self) -> u16 {
        self.scroll_x() & 0x07
    }

    pub fn scroll_x_coarse(&self) -> u16 {
        self.scroll_x() >> 3
    }

    pub fn scroll_y(&self) -> u16 {
        (self.ppuscroll & 0x00FF) >> 0
    }

    pub fn scroll_y_coarse(&self) -> u16 {
        self.scroll_y() >> 3
    }

    pub fn incr_ppuaddr(&mut self) {
        let incr_size = self.ppuctrl.vram_addr_step();
        self.ppuaddr = self.ppuaddr.wrapping_add(incr_size);
    }

    pub fn incr_oamaddr(&mut self) {
        self.oamaddr = self.oamaddr.wrapping_add(1);
    }
}

fn write_addr_byte(latch: &mut AddrByte, target: &mut u16, val: u8) {
    match *latch {
        AddrByte::High => {
            *target = (*target & 0x00FF) | ((val as u16) << 8);
            *latch = AddrByte::Low;
        }
        AddrByte::Low => {
            *target = (*target & 0xFF00) | ((val as u16) << 0);
            *latch = AddrByte::High;
        }
    }
}

impl Default for PPUReg {
    fn default() -> PPUReg {
        PPUReg {
            ppuctrl: PPUCtrl::empty(),
            ppumask: PPUMask::empty(),
            ppustat: PPUStat::empty(),
            oamaddr: 0,
            ppuscroll: 0,
            ppuaddr: 0,
            dyn_latch: 0,
            address_latch: AddrByte::High,
        }
    }
}

impl MemSegment for PPUReg {
    fn read(&mut self, idx: u16) -> u8 {
        match idx % 8 {
            0x0000 => self.dyn_latch,
            0x0001 => self.dyn_latch,
            0x0002 => {
                self.address_latch = AddrByte::High;
                let res = self.ppustat.bits | (self.dyn_latch & 0b0001_1111);
                self.ppustat.remove(VBLANK);
                res
            }
            0x0003 => self.dyn_latch,
            0x0005 => self.dyn_latch,
            0x0006 => self.dyn_latch,
            _ => invalid_address!(idx),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        self.dyn_latch = val;
        match idx % 8 {
            0x0000 => self.ppuctrl = PPUCtrl::new(val),
            0x0001 => self.ppumask = PPUMask::from_bits_truncate(val),
            0x0002 => (),
            0x0003 => self.oamaddr = val,
            0x0005 => write_addr_byte(&mut self.address_latch, &mut self.ppuscroll, val),
            0x0006 => write_addr_byte(&mut self.address_latch, &mut self.ppuaddr, val),
            _ => invalid_address!(idx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory::MemSegment;
    use ppu::PPU;
    use ppu::tests::*;

    fn assert_register_single_writable(idx: u16, getter: &Fn(&PPU) -> u8) {
        let mut ppu = create_test_ppu();
        ppu.write(idx, 12);
        assert_eq!(getter(&ppu), 12);
        ppu.write(idx, 125);
        assert_eq!(getter(&ppu), 125);
    }

    fn assert_register_double_writable(idx: u16, getter: &Fn(&PPU) -> u16) {
        let mut ppu = create_test_ppu();
        ppu.write(idx, 0xDE);
        assert_eq!(getter(&ppu), 0xDE00);
        assert_eq!(AddrByte::Low, ppu.reg.address_latch);
        ppu.write(idx, 0xAD);
        assert_eq!(getter(&ppu), 0xDEAD);
        assert_eq!(AddrByte::High, ppu.reg.address_latch);
    }

    fn assert_register_ignores_writes(idx: u16, getter: &Fn(&PPU) -> u8) {
        let mut ppu = create_test_ppu();
        ppu.write(idx, 12);
        assert_eq!(getter(&ppu), 0);
        ppu.write(idx, 125);
        assert_eq!(getter(&ppu), 0);
    }

    fn assert_writing_register_fills_latch(idx: u16) {
        let mut ppu = create_test_ppu();
        ppu.write(idx, 12);
        assert_eq!(ppu.reg.dyn_latch, 12);
        ppu.write(idx, 125);
        assert_eq!(ppu.reg.dyn_latch, 125);
    }

    fn assert_register_is_readable(idx: u16, setter: &Fn(&mut PPU, u8) -> ()) {
        let mut ppu = create_test_ppu();
        setter(&mut ppu, 12);
        assert_eq!(ppu.read(idx), 12);
        setter(&mut ppu, 125);
        assert_eq!(ppu.read(idx), 125);
    }

    fn assert_register_not_readable(idx: u16) {
        let mut ppu = create_test_ppu();
        ppu.reg.dyn_latch = 12;
        assert_eq!(ppu.read(idx), 12);
        ppu.reg.dyn_latch = 125;
        assert_eq!(ppu.read(idx), 125);
    }

    #[test]
    fn ppuctrl_is_write_only_register() {
        assert_register_single_writable(0x2000, &|ref ppu| ppu.reg.ppuctrl.bits);
        assert_writing_register_fills_latch(0x2000);
        assert_register_not_readable(0x2000);
    }

    #[test]
    fn ppu_mirrors_address() {
        assert_register_single_writable(0x2008, &|ref ppu| ppu.reg.ppuctrl.bits);
        assert_register_single_writable(0x2010, &|ref ppu| ppu.reg.ppuctrl.bits);
    }

    #[test]
    fn ppumask_is_write_only_register() {
        assert_register_single_writable(0x2001, &|ref ppu| ppu.reg.ppumask.bits());
        assert_writing_register_fills_latch(0x2001);
        assert_register_not_readable(0x2001);
    }

    #[test]
    fn ppustat_is_read_only_register() {
        assert_register_ignores_writes(0x2002, &|ref ppu| ppu.reg.ppustat.bits);
        assert_writing_register_fills_latch(0x2002);
        assert_register_is_readable(0x2002,
                                    &|ref mut ppu, val| {
                                        ppu.reg.ppustat = PPUStat::from_bits_truncate(val);
                                        ppu.reg.dyn_latch = val;
                                    });
    }

    #[test]
    fn reading_ppustat_returns_part_of_dynlatch() {
        let mut ppu = create_test_ppu();
        ppu.reg.dyn_latch = 0b0001_0101;
        ppu.reg.ppustat = PPUStat::from_bits_truncate(0b1010_0101);
        assert_eq!(ppu.read(0x2002), 0b1011_0101);
    }

    #[test]
    fn reading_ppustat_clears_addr_latch() {
        let mut ppu = create_test_ppu();
        ppu.reg.address_latch = AddrByte::Low;
        ppu.read(0x2002);
        assert_eq!(ppu.reg.address_latch, AddrByte::High);
    }

    #[test]
    fn oamaddr_is_write_only_register() {
        assert_register_single_writable(0x2003, &|ref ppu| ppu.reg.oamaddr);
        assert_writing_register_fills_latch(0x2003);
        assert_register_not_readable(0x2003);
    }

    #[test]
    fn ppuscroll_is_2x_write_only_register() {
        assert_register_double_writable(0x2005, &|ref ppu| ppu.reg.ppuscroll);
        assert_writing_register_fills_latch(0x2005);
        assert_register_not_readable(0x2005);
    }

    #[test]
    fn ppuaddr_is_2x_write_only_register() {
        assert_register_double_writable(0x2006, &|ref ppu| ppu.reg.ppuaddr);
        assert_writing_register_fills_latch(0x2006);
        assert_register_not_readable(0x2006);
    }
}
