use ::memory::MemSegment;

bitflags! {
    flags OAMAttr : u8 {
        const FLIP_VERT = 0b1000_0000,
        const FLIP_HORZ = 0b0100_0000,
        const BEHIND    = 0b0010_0000,
        const PALETTE1  = 0b0000_0010,
        const PALETTE2  = 0b0000_0001,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OAMEntry {
    y: u8,
    tile: u8,
    attr: OAMAttr,
    x: u8,
}

impl OAMEntry {
    pub fn zero() -> OAMEntry {
        OAMEntry::new(0, 0, 0, 0)
    }

    pub fn new(y: u8, tile: u8, attr: u8, x: u8) -> OAMEntry {
        OAMEntry {
            y: y,
            tile: tile,
            attr: OAMAttr::from_bits_truncate(attr),
            x: x,
        }
    }
}

impl MemSegment for OAMEntry {
    fn read(&mut self, idx: u16) -> u8 {
        match idx % 4 {
            0 => self.y,
            1 => self.tile,
            2 => self.attr.bits(),
            3 => self.x,
            _ => panic!("Math is broken!"),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        match idx % 4 {
            0 => self.y = val,
            1 => self.tile = val,
            2 => self.attr = OAMAttr::from_bits_truncate(val),
            3 => self.x = val,
            _ => panic!("Math is broken!"),
        }
    }
}

#[cfg(tests)]
mod tests {
    use memory::MemSegment;
    use ppu::PPU;
    use super::*;
    
    #[test]
    fn reading_oamdata_uses_oamaddr_as_index_into_oam() {
        let mut ppu = create_test_ppu();
        for x in 0u8..63u8 {
            ppu.oam[x as usize] = OAMEntry::new(x, x, x, x);
        }
        ppu.reg.oamaddr = 0;
        assert_eq!(ppu.read(0x2004), 0);
        ppu.reg.oamaddr = 10;
        assert_eq!(ppu.read(0x2004), 2);
    }
    
    #[test]
    fn writing_oamdata_uses_oamaddr_as_index_into_oam() {
        let mut ppu = create_test_ppu();
        ppu.reg.oamaddr = 0;
        ppu.write(0x2004, 12);
        assert_eq!(ppu.oam[0].y, 12);
        ppu.reg.oamaddr = 10;
        ppu.write(0x2004, 3);
        assert_eq!(ppu.oam[2].attr.bits(), 3);
    }
}