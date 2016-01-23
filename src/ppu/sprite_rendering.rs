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
struct OAMEntry {
    y: u8,
    tile: u8,
    attr: OAMAttr,
    x: u8,
}

impl Default for OAMEntry {
    fn default() -> OAMEntry {
        OAMEntry {
            y: 0,
            tile: 0,
            attr: OAMAttr::from_bits_truncate(0),
            x: 0,
        }
    }
}

impl MemSegment for OAMEntry {
    fn read(&mut self, idx: u16) -> u8 {
        match idx {
            0 => self.y,
            1 => self.tile,
            2 => self.attr.bits(),
            3 => self.x,
            x => invalid_address!(x),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        match idx {
            0 => self.y = val,
            1 => self.tile = val,
            2 => self.attr = OAMAttr::from_bits_truncate(val),
            3 => self.x = val,
            x => invalid_address!(x),
        }
    }
}

pub struct SpriteRenderingData {
    primary_oam: [OAMEntry; 64],
}

impl Default for SpriteRenderingData {
    fn default() -> SpriteRenderingData {
        SpriteRenderingData {
            primary_oam: [Default::default(); 64]
        }
    }
}

///Reads the primary OAM table. 
impl MemSegment for SpriteRenderingData {
    fn read(&mut self, idx: u16) -> u8 {
        if idx > 256 {
            invalid_address!(idx);
        }
        self.primary_oam[idx as usize / 4].read( idx % 4 )
    }

    fn write(&mut self, idx: u16, val: u8) {
        if idx > 256 {
            invalid_address!(idx);
        }
        self.primary_oam[idx as usize / 4].write( idx % 4, val )
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