use super::PaletteIndex;
use super::PaletteSet;
use super::SCREEN_BUFFER_SIZE;
use super::SCREEN_HEIGHT;
use super::SCREEN_WIDTH;
use super::TilePattern;
use super::ppu_memory::PPUMemory;
use super::ppu_reg::PPUReg;
use memory::MemSegment;
use std::cmp;

bitflags! {
    struct OAMAttr : u8 {
        const FLIP_VERT = 0b1000_0000;
        const FLIP_HORZ = 0b0100_0000;
        const BEHIND    = 0b0010_0000;
        #[allow(dead_code)]
        const PALETTE   = 0b0000_0011;
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
    fn is_on_scanline(&self, scanline: u16, sprite_height: u16) -> bool {
        self.y <= scanline && scanline < self.y + sprite_height
    }

    fn build_details(
        &self,
        idx: usize,
        sl: u16,
        reg: &PPUReg,
        mem: &mut PPUMemory,
    ) -> SpriteDetails {
        let tile_id = self.tile;
        let fine_y_scroll = get_fine_scroll(
            reg.ppuctrl.sprite_height(),
            sl,
            self.y,
            self.attr.contains(FLIP_VERT),
        );
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
    fn do_get_pixel(&self, x: u16) -> PaletteIndex {
        let fine_x = get_fine_scroll(8, x, self.x as u16, self.attr.contains(FLIP_HORZ));
        let attr = self.attr;
        let color_id = self.tile.get_color_in_pattern(fine_x as u32);
        PaletteIndex::from_unpacked(PaletteSet::Sprite, attr.palette(), color_id)
    }

    #[allow(needless_range_loop)] // false positive - we need x to get the pixels.
    fn hit_test(
        &self,
        original_pixel_line: &[PaletteIndex; SCREEN_WIDTH],
        reg: &mut PPUReg,
        intersection: &Interval,
    ) {
        for x in intersection.start..(intersection.end) {
            if !original_pixel_line[x].is_transparent() &&
                !self.do_get_pixel(x as u16).is_transparent()
            {
                reg.ppustat.insert(::ppu::ppu_reg::SPRITE_0);
                return;
            }
        }
    }

    fn blit(
        &self,
        original_pixel_line: &[PaletteIndex; SCREEN_WIDTH],
        pixel_line: &mut [PaletteIndex],
        intersection: &Interval,
    ) {
        for x in intersection.start..(intersection.end) {
            pixel_line[x] = self.mix(
                original_pixel_line[x],
                pixel_line[x],
                self.do_get_pixel(x as u16),
            )
        }
    }

    fn mix(
        &self,
        background: PaletteIndex,
        current: PaletteIndex,
        sprite: PaletteIndex,
    ) -> PaletteIndex {
        if !sprite.is_transparent() && !self.attr.priority() && !background.is_transparent() {
            background
        } else if current.is_transparent() || (self.attr.priority() && !sprite.is_transparent()) {
            sprite
        } else {
            current
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
    primary_oam: Box<[OAMEntry; 64]>,
    secondary_oam: Box<[[SpriteDetails; 8]; SCREEN_HEIGHT]>,
}

impl Default for SpriteRenderer {
    fn default() -> SpriteRenderer {
        SpriteRenderer {
            primary_oam: Box::new([Default::default(); 64]),
            secondary_oam: Box::new([[Default::default(); 8]; SCREEN_HEIGHT]),
        }
    }
}

fn get_fine_scroll(size: u16, screen_dist: u16, sprite_dist: u16, flip: bool) -> u16 {
    let scroll = screen_dist - sprite_dist;
    if flip { (size - 1) - scroll } else { scroll }
}

impl SpriteRenderer {
    pub fn render(
        &mut self,
        buffer: &mut [PaletteIndex; SCREEN_BUFFER_SIZE],
        reg: &mut PPUReg,
        start: usize,
        stop: usize,
    ) {
        self.draw(buffer, reg, start, stop)
    }

    pub fn sprite_eval(&mut self, scanline: u16, reg: &PPUReg, mem: &mut PPUMemory) {
        if scanline + 1 >= SCREEN_HEIGHT as u16 {
            return;
        }
        let mut n = 0;
        let sprite_height = reg.ppuctrl.sprite_height();
        let secondary_oam_line = &mut self.secondary_oam[scanline as usize + 1];
        secondary_oam_line.copy_from_slice(&EMPTY_SECONDARY_OAM_LINE);
        for x in 0..64 {
            let oam = &self.primary_oam[x];
            if oam.is_on_scanline(scanline, sprite_height) {
                secondary_oam_line[n] = oam.build_details(x, scanline, reg, mem);
                n += 1;
                if n == 8 {
                    return;
                }
            }
        }
    }

    fn draw(
        &mut self,
        buffer: &mut [PaletteIndex; SCREEN_BUFFER_SIZE],
        reg: &mut PPUReg,
        start: usize,
        stop: usize,
    ) {
        let mut current_scanline = start / SCREEN_WIDTH;
        let mut last_scanline_boundary = current_scanline * SCREEN_WIDTH;
        let next_scanline = current_scanline + 1;
        let mut next_scanline_boundary = next_scanline * SCREEN_WIDTH;

        let mut current = start;
        while current < stop {
            let segment_start = current - last_scanline_boundary;
            let segment_end = cmp::min(next_scanline_boundary, stop) - last_scanline_boundary;

            self.render_segment(
                buffer,
                reg,
                current_scanline,
                last_scanline_boundary,
                next_scanline_boundary,
                segment_start,
                segment_end,
            );
            current_scanline += 1;
            last_scanline_boundary = next_scanline_boundary;
            current = next_scanline_boundary;
            next_scanline_boundary += SCREEN_WIDTH;
        }
    }

    #[allow(too_many_arguments)]
    fn render_segment(
        &mut self,
        buffer: &mut [PaletteIndex; SCREEN_BUFFER_SIZE],
        reg: &mut PPUReg,
        scanline: usize,
        line_start: usize,
        line_stop: usize,
        start: usize,
        stop: usize,
    ) {
        let oam_line = &self.secondary_oam[scanline];
        let pixel_line = &mut buffer[line_start..line_stop];

        let mut original_pixel_line: [PaletteIndex; SCREEN_WIDTH] =
            unsafe { ::std::mem::uninitialized() };
        original_pixel_line.copy_from_slice(pixel_line);

        let segment = Interval::new(start, stop);

        for sprite in oam_line.iter().rev() {
            let sprite_interval = Interval::new(sprite.x as usize, sprite.x as usize + 8);
            if segment.intersects_with(&sprite_interval) {
                let intersection = segment.intersection(&sprite_interval);
                if sprite.idx == 0 {
                    sprite.hit_test(&original_pixel_line, reg, &intersection);
                }
                sprite.blit(&original_pixel_line, pixel_line, &intersection);
            }
        }
    }

    #[cfg(feature = "debug_features")]
    pub fn mouse_pick(&self, px_x: i32, px_y: i32) {
        let scanline = px_y as usize;
        let pixel = px_x as u16;
        for sprite in &self.secondary_oam[scanline] {
            let spr_x = sprite.x as u16;
            if spr_x <= pixel && pixel <= (spr_x + 8) {
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
    use super::*;
    use memory::MemSegment;
    use ppu::PPU;

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
