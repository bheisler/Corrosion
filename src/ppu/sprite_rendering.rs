use memory::MemSegment;
use super::PaletteIndex;
use super::PaletteSet;
use super::TilePattern;
use super::SCREEN_BUFFER_SIZE;
use super::SCREEN_WIDTH;
use super::SCREEN_HEIGHT;
use super::TRANSPARENT;
use super::ppu_reg::PPUReg;
use super::ppu_memory::PPUMemory;
use std::cmp;

bitflags! {
    flags OAMAttr : u8 {
        const FLIP_VERT = 0b1000_0000,
        const FLIP_HORZ = 0b0100_0000,
        const BEHIND    = 0b0010_0000,
        #[allow(dead_code)]
        const PALETTE   = 0b0000_0011,
    }
}

impl OAMAttr {
    fn palette(&self) -> u8 {
        self.bits & 0x03
    }

    fn priority(&self) -> bool {
        !self.contains(BEHIND)
    }
}

#[derive(Debug, Copy, Clone)]
struct OAMEntry {
    y: u16,
    tile: u8,
    attr: OAMAttr,
    x: u8,
}

impl OAMEntry {
    fn is_on_scanline(&self, reg: &PPUReg, scanline: u16) -> bool {
        self.y <= scanline && scanline < self.y + reg.ppuctrl.sprite_height()
    }

    fn build_details(&self,
                     idx: usize,
                     sl: u16,
                     reg: &PPUReg,
                     mem: &mut PPUMemory)
                     -> SpriteDetails {
        let tile_id = self.tile;
        let fine_y_scroll = get_fine_scroll(reg.ppuctrl.sprite_height(),
                                            sl,
                                            self.y,
                                            self.attr.contains(FLIP_VERT));
        let tile = if reg.ppuctrl.tall_sprites() {
            let tile_table = (tile_id as u16 & 0b0000_0001) << 12;
            let mut tile_id = tile_id & 0b1111_1110;
            let mut fine_y_scroll = fine_y_scroll;
            if fine_y_scroll >= 8 {
                tile_id += 1;
                fine_y_scroll -= 8;
            }
            mem.read_tile_pattern(tile_id, fine_y_scroll, tile_table)
        } else {
            let tile_table = reg.ppuctrl.sprite_table();
            mem.read_tile_pattern(tile_id, fine_y_scroll, tile_table)
        };
        SpriteDetails {
            idx: idx,
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
            0 => self.y as u8,
            1 => self.tile,
            2 => self.attr.bits(),
            3 => self.x,
            x => invalid_address!(x),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        match idx {
            0 => self.y = val as u16,
            1 => self.tile = val,
            2 => self.attr = OAMAttr::from_bits_truncate(val),
            3 => self.x = val,
            x => invalid_address!(x),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct SpriteDetails {
    idx: usize,
    x: u8,
    attr: OAMAttr,
    tile: TilePattern,
}
const NO_SPRITE: SpriteDetails = SpriteDetails {
    idx: 0xFF,
    x: 0xFF,
    attr: OAMAttr { bits: 0 },
    tile: ::ppu::NO_TILE,
};
const EMPTY_SECONDARY_OAM_LINE: [SpriteDetails; 8] = [NO_SPRITE; 8];

impl SpriteDetails {
    fn do_get_pixel(&self, x: u16) -> (bool, PaletteIndex) {
        let fine_x = get_fine_scroll(8, x, self.x as u16, self.attr.contains(FLIP_HORZ));
        let attr = self.attr;
        let color_id = self.tile.get_color_in_pattern(fine_x as u32);
        let idx = PaletteIndex::from_unpacked(PaletteSet::Sprite, attr.palette(), color_id);
        (attr.priority(), idx)
    }

    fn blit(&self,
            pixel_line: &mut [PaletteIndex],
            priority_line: &mut [bool],
            sprite0_line: &mut [bool],
            segment: &Interval,
            sprite_interval: &Interval) {
        let intersection = segment.intersection(sprite_interval);
        for x in intersection.start..(intersection.end) {
            let (pri, pal) = self.do_get_pixel(x as u16);
            if !pal.is_transparent() {
                pixel_line[x] = pal;
                priority_line[x] = pri;
                sprite0_line[x] = self.idx == 0;
            }
        }
    }
}

impl Default for SpriteDetails {
    fn default() -> SpriteDetails {
        NO_SPRITE
    }
}

#[derive(Debug)]
struct Interval {
    start: usize,
    end: usize,
}

impl Interval {
    fn new(start: usize, end: usize) -> Interval {
        Interval {
            start: start,
            end: end,
        }
    }

    fn intersects_with(&self, other: &Interval) -> bool {
        self.start < other.end && self.end > other.start
    }

    fn intersection(&self, other: &Interval) -> Interval {
        Interval {
            start: cmp::max(self.start, other.start),
            end: cmp::min(self.end, other.end),
        }
    }
}

pub struct SpriteRenderer {
    primary_oam: [OAMEntry; 64],
    secondary_oam: [[SpriteDetails; 8]; SCREEN_HEIGHT],

    pixel_buffer: Box<[PaletteIndex; SCREEN_BUFFER_SIZE]>,
    priority_buffer: Box<[bool; SCREEN_BUFFER_SIZE]>,
    sprite0_buffer: Box<[bool; SCREEN_BUFFER_SIZE]>,
}

impl Default for SpriteRenderer {
    fn default() -> SpriteRenderer {
        SpriteRenderer {
            primary_oam: [Default::default(); 64],
            secondary_oam: [[Default::default(); 8]; SCREEN_HEIGHT],

            pixel_buffer: Box::new([TRANSPARENT; SCREEN_BUFFER_SIZE]),
            priority_buffer: Box::new([false; SCREEN_BUFFER_SIZE]),
            sprite0_buffer: Box::new([false; SCREEN_BUFFER_SIZE]),
        }
    }
}

fn get_fine_scroll(size: u16, screen_dist: u16, sprite_dist: u16, flip: bool) -> u16 {
    let scroll = screen_dist - sprite_dist;
    if flip {
        (size - 1) - scroll
    } else {
        scroll
    }
}

impl SpriteRenderer {
    pub fn render(&mut self, start: usize, stop: usize) {
        self.draw(start, stop)
    }

    pub fn run_cycle(&mut self, cyc: u16, sl: i16, reg: &mut PPUReg, mem: &mut PPUMemory) {
        if let (0, sl @ 0...239) = (cyc, sl) {
            self.sprite_eval(sl as u16, reg, mem);
        }
    }

    fn sprite_eval(&mut self, scanline: u16, reg: &PPUReg, mem: &mut PPUMemory) {
        if scanline + 1 >= SCREEN_HEIGHT as u16 {
            return;
        }
        let mut n = 0;
        let secondary_oam_line = &mut self.secondary_oam[scanline as usize + 1];
        secondary_oam_line.copy_from_slice(&EMPTY_SECONDARY_OAM_LINE);
        for x in 0..64 {
            let oam = &self.primary_oam[x];
            if oam.is_on_scanline(reg, scanline) {
                secondary_oam_line[n] = oam.build_details(x, scanline, reg, mem);
                n += 1;
                if n == 8 {
                    return;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        for dest in self.pixel_buffer.iter_mut() {
            *dest = TRANSPARENT;
        }
        for dest in self.priority_buffer.iter_mut() {
            *dest = false;
        }
        for dest in self.sprite0_buffer.iter_mut() {
            *dest = false;
        }
    }

    fn draw(&mut self, start: usize, stop: usize) {
        let mut current_scanline = start / SCREEN_WIDTH;
        let mut last_scanline_boundary = current_scanline * SCREEN_WIDTH;
        let next_scanline = current_scanline + 1;
        let mut next_scanline_boundary = next_scanline * SCREEN_WIDTH;

        let mut current = start;
        while current < stop {
            let segment_start = current - last_scanline_boundary;
            let segment_end = cmp::min(next_scanline_boundary, stop) - last_scanline_boundary;

            self.render_segment(current_scanline,
                                last_scanline_boundary,
                                next_scanline_boundary,
                                segment_start,
                                segment_end);
            current_scanline += 1;
            last_scanline_boundary = next_scanline_boundary;
            current = next_scanline_boundary;
            next_scanline_boundary += SCREEN_WIDTH;
        }
    }

    fn render_segment(&mut self,
                      scanline: usize,
                      line_start: usize,
                      line_stop: usize,
                      start: usize,
                      stop: usize) {
        let oam_line = &self.secondary_oam[scanline];
        let pixel_line = &mut self.pixel_buffer[line_start..line_stop];
        let priority_line = &mut self.priority_buffer[line_start..line_stop];
        let sprite0_line = &mut self.sprite0_buffer[line_start..line_stop];

        let segment = Interval::new(start, stop);

        for sprite in oam_line.iter().rev() {
            let sprite_interval = Interval::new(sprite.x as usize, sprite.x as usize + 8);
            if segment.intersects_with(&sprite_interval) {
                sprite.blit(pixel_line,
                            priority_line,
                            sprite0_line,
                            &segment,
                            &sprite_interval);
            }
        }
    }

    pub fn buffers(&self)
                   -> (&[PaletteIndex; SCREEN_BUFFER_SIZE],
                       &[bool; SCREEN_BUFFER_SIZE],
                       &[bool; SCREEN_BUFFER_SIZE]) {
        (&self.pixel_buffer, &self.priority_buffer, &self.sprite0_buffer)
    }

    #[cfg(feature="mousepick")]
    pub fn mouse_pick(&self, px_x: i32, px_y: i32) {
        let scanline = px_y as usize;
        let pixel = px_x as u8;
        for sprite in &self.secondary_oam[scanline] {
            if sprite.x <= pixel && pixel <= (sprite.x + 8) {
                println!("{:?}", sprite);
            }
        }
    }
}

/// Reads the primary OAM table.
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
