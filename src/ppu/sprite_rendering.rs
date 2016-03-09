use memory::MemSegment;
use super::PaletteIndex;
use super::PaletteSet;
use super::TilePattern;
use super::SCREEN_BUFFER_SIZE;
use super::SCREEN_WIDTH;
use super::SCREEN_HEIGHT;
use super::ppu_reg::PPUReg;
use super::ppu_memory::PPUMemory;

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

impl OAMEntry {
    fn is_on_scanline(&self, scanline: u16) -> bool {
        let y = self.y as u16;
        y <= scanline && scanline < y + 8
    }

    fn build_details(&self, sl: u16, reg: &PPUReg, mem: &mut PPUMemory) -> SpriteDetails {
        let tile_id = self.tile;
        let fine_y_scroll = get_fine_scroll(sl, self.y as u16, self.attr.contains(FLIP_VERT));
        let tile_table = reg.ppuctrl.sprite_table();
        let tile = mem.read_tile_pattern(tile_id, fine_y_scroll, tile_table);
        SpriteDetails {
            x: self.x,
            attr: self.attr,
            tile: tile,
        }
    }
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

impl SpriteDetails {
    fn is_active(&self, x: u16) -> bool {
        x.wrapping_sub(self.x as u16) < 8
    }

    fn do_get_pixel(&self, x: u16) -> (SpritePriority, PaletteIndex) {
        let fine_x = get_fine_scroll(x, self.x as u16, self.attr.contains(FLIP_HORZ));
        let attr = self.attr;
        let color_id = self.tile.get_color_in_pattern(fine_x as u32);
        let idx = PaletteIndex {
            set: PaletteSet::Sprite,
            palette_id: attr.palette(),
            color_id: color_id,
        };
        return (attr.priority(), idx);
    }
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
    secondary_oam: [[SpriteDetails; 8]; SCREEN_HEIGHT],

    pixel_buffer: Box<[PaletteIndex]>,
    priority_buffer: Box<[SpritePriority]>,
}

impl Default for SpriteRenderer {
    fn default() -> SpriteRenderer {
        SpriteRenderer {
            primary_oam: [Default::default(); 64],
            secondary_oam: [[Default::default(); 8]; SCREEN_HEIGHT],

            pixel_buffer: vec![Default::default(); SCREEN_BUFFER_SIZE].into_boxed_slice(),
            priority_buffer: vec![SpritePriority::Background; SCREEN_BUFFER_SIZE]
                                 .into_boxed_slice(),
        }
    }
}

///Computes the next scanline boundary after the given pixel.
fn pixel_to_scanline(px: usize) -> usize {
    (px + SCREEN_WIDTH - 1) / SCREEN_WIDTH
}

fn get_fine_scroll(screen_dist: u16, sprite_dist: u16, flip: bool) -> u16 {
    let scroll = screen_dist - sprite_dist;
    if flip {
        7 - scroll
    } else {
        scroll
    }
}

impl SpriteRenderer {
    pub fn render(&mut self, start: usize, stop: usize, reg: &PPUReg, mem: &mut PPUMemory) {
        let start_sl = pixel_to_scanline(start);
        let stop_sl = pixel_to_scanline(stop);

        for sl in start_sl..stop_sl {
            self.sprite_eval(sl as u16, reg, mem)
        }

        self.draw(start, stop)
    }

    // TODO: Optimize this.
    fn sprite_eval(&mut self, scanline: u16, reg: &PPUReg, mem: &mut PPUMemory) {
        if scanline + 1 >= SCREEN_HEIGHT as u16 {
            return;
        }
        let mut n = 0;
        let secondary_oam_line = &mut self.secondary_oam[scanline as usize + 1];
        *secondary_oam_line = [Default::default(); 8];
        for x in 0..64 {
            let oam = &self.primary_oam[x];
            if oam.is_on_scanline(scanline) {
                secondary_oam_line[n] = oam.build_details(scanline, reg, mem);
                n += 1;
                if n == 8 {
                    return;
                }
            }
        }
    }

    fn draw(&mut self, start: usize, stop: usize) {
        let mut next_scanline = (start / SCREEN_WIDTH) + 1;
        let mut scanline_boundary = next_scanline * SCREEN_WIDTH;
        let mut line = &self.secondary_oam[next_scanline - 1];

        for pixel in start..stop {
            let x = (pixel % SCREEN_WIDTH) as u16;

            let (pri, pal) = line.iter()
                                 .filter(|sprite| sprite.is_active(x))
                                 .map(|sprite| sprite.do_get_pixel(x))
                                 .filter(|pixel| !pixel.1.is_transparent())
                                 .next()
                                 .unwrap_or((SpritePriority::Background, TRANSPARENT));
            self.pixel_buffer[pixel] = pal;
            self.priority_buffer[pixel] = pri;

            if pixel == scanline_boundary {
                next_scanline = (pixel / SCREEN_WIDTH) + 1;
                scanline_boundary = next_scanline * SCREEN_WIDTH;
                line = &self.secondary_oam[next_scanline - 1];
            }
        }
    }

    pub fn buffers(&self) -> (&[PaletteIndex], &[SpritePriority]) {
        (&self.pixel_buffer, &self.priority_buffer)
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
