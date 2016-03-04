use memory::MemSegment;
use super::PPU;
use super::PaletteIndex;
use super::PaletteSet;
use super::TilePattern;

const TRANSPARENT: PaletteIndex = PaletteIndex {
    set: PaletteSet::Sprite,
    palette_id: 0,
    color_id: 0,
};

bitflags! {
    flags OAMAttr : u8 {
        const FLIP_VERT = 0b1000_0000,
        const FLIP_HORZ = 0b0100_0000,
        const BEHIND    = 0b0010_0000,
        const PALETTE1  = 0b0000_0010,
        const PALETTE2  = 0b0000_0001,
    }
}

impl OAMAttr {
    fn palette(&self) -> u8 {
        self.bits & 0x03
    }

    fn priority(&self) -> SpritePriority {
        if self.contains(BEHIND) {
            SpritePriority::Background
        } else {
            SpritePriority::Foreground
        }
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
            y: 0xFF,
            tile: 0,
            attr: OAMAttr::from_bits_truncate(0),
            x: 0xFF,
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

#[derive(Debug, Copy, Clone)]
struct SpriteDetails {
    x: u8,
    attr: OAMAttr,
    tile: TilePattern,
}

impl Default for SpriteDetails {
    fn default() -> SpriteDetails {
        SpriteDetails {
            x: 0xFF,
            attr: OAMAttr::empty(),
            tile: Default::default(),
        }
    }
}

pub struct SpriteRenderer {
    primary_oam: [OAMEntry; 64],
    secondary_oam: [SpriteDetails; 8],
}

impl Default for SpriteRenderer {
    fn default() -> SpriteRenderer {
        SpriteRenderer {
            primary_oam: [Default::default(); 64],
            secondary_oam: [Default::default(); 8],
        }
    }
}

impl SpriteRenderer {
    pub fn run(&mut self, start: u64, stop: u64) {
        //TODO: Not implemented yet.
    }
    
    pub fn render(&mut self, start: usize, stop: usize) {
        //TODO: Not implemented yet.
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SpritePriority {
    Foreground,
    Background,
}

///Reads the primary OAM table. 
impl MemSegment for SpriteRenderer {
    fn read(&mut self, idx: u16) -> u8 {
        if idx > 256 {
            invalid_address!(idx);
        }
        self.primary_oam[idx as usize / 4].read(idx % 4)
    }

    fn write(&mut self, idx: u16, val: u8) {
        if idx > 256 {
            invalid_address!(idx);
        }
        self.primary_oam[idx as usize / 4].write(idx % 4, val)
    }
}

impl PPU {
    pub fn visible_scanline_sprite(&mut self, pixel: u16, scanline: u16) {
        if pixel == 0 {
            self.sprite_eval(scanline);
        }
    }

    fn sprite_eval(&mut self, scanline: u16) {
        let mut n = 0;
        self.sprite_data.secondary_oam = [Default::default(); 8];
        for x in 0..64 {
            let oam = self.sprite_data.primary_oam[x];
            if self.is_on_scanline(oam, scanline) {
                self.sprite_data.secondary_oam[n] = self.convert_oam_entry(oam, scanline);
                n += 1;
                if n == 8 {
                    return;
                }
            }
        }
    }

    fn is_on_scanline(&self, oam: OAMEntry, scanline: u16) -> bool {
        let y = oam.y as u16;
        y <= scanline && scanline < y + 8
    }

    fn convert_oam_entry(&mut self, oam: OAMEntry, sl: u16) -> SpriteDetails {
        let tile_id = oam.tile;
        let fine_y_scroll = PPU::get_fine_scroll(sl, oam.y as u16, oam.attr.contains(FLIP_VERT));
        let tile_table = self.reg.ppuctrl.sprite_table();
        let tile = self.read_tile_pattern(tile_id, fine_y_scroll, tile_table);
        SpriteDetails {
            x: oam.x,
            attr: oam.attr,
            tile: tile,
        }
    }

    pub fn get_sprite_pixel(&mut self, x: u16) -> (SpritePriority, PaletteIndex) {
        for n in 0..8 {
            let det_x = self.sprite_data.secondary_oam[n];
            if self.is_active(det_x, x) {
                let pixel = self.do_get_pixel(det_x, x);
                if !pixel.1.is_transparent() {
                    return pixel;
                }
            }
        }
        return (SpritePriority::Background, TRANSPARENT);
    }

    fn is_active(&self, details: SpriteDetails, x: u16) -> bool {
        x.wrapping_sub(details.x as u16) < 8
    }

    fn get_fine_scroll(screen_dist: u16, sprite_dist: u16, flip: bool) -> u16 {
        let scroll = screen_dist - sprite_dist;
        if flip {
            7 - scroll
        } else {
            scroll
        }
    }

    fn do_get_pixel(&mut self, details: SpriteDetails, x: u16) -> (SpritePriority, PaletteIndex) {
        let fine_x = PPU::get_fine_scroll(x, details.x as u16, details.attr.contains(FLIP_HORZ));
        let attr = details.attr;
        let color_id = self.get_color_in_pattern(details.tile, fine_x as u32);
        let idx = PaletteIndex {
            set: PaletteSet::Sprite,
            palette_id: attr.palette(),
            color_id: color_id,
        };
        return (attr.priority(), idx);
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

    #[test]
    fn test_sprite_on_scanline() {
        let mut ppu = create_test_ppu();
        let mut oam: OAMEntry = Default::default();
        oam.y = 10;

        assert!(!ppu.is_on_scanline(oam, 9));
        for sl in 10..18 {
            assert!(ppu.is_on_scanline(oam, sl));
        }
        assert!(!ppu.is_on_scanline(oam, 18));
    }
}
