#![allow(dead_code, unused_variables)] //Temporary

use super::PaletteIndex;
use super::PaletteSet;
use super::TilePattern;
use super::SCREEN_BUFFER_SIZE;
use super::SCREEN_WIDTH;
use super::SCREEN_HEIGHT;
use super::ppu_reg::PPUReg;
use super::ppu_memory::PPUMemory;
use memory::MemSegment;
use std::cmp;

const TILES_PER_LINE: usize = 34;

const TRANSPARENT: PaletteIndex = PaletteIndex {
    set: PaletteSet::Background,
    palette_id: 0,
    color_id: 0,
};

const EMPTY_PIXEL_BUFFER: [PaletteIndex; SCREEN_BUFFER_SIZE] = [TRANSPARENT; SCREEN_BUFFER_SIZE];

#[derive(Debug, Copy, Clone, PartialEq)]
struct TileAttribute {
    bits: u8,
}

impl TileAttribute {
    fn new(bits: u8) -> TileAttribute {
        TileAttribute { bits: bits }
    }

    fn get_palette(&self, x: u16, y: u16) -> u8 {
        let y = y % SCREEN_HEIGHT as u16;
        let mut at = self.bits;
        if y & 0x10 != 0 {
            at >>= 4
        }
        if x & 0x10 != 0 {
            at >>= 2
        }
        at & 0x03
    }

    #[cfg(feature="mousepick")]
    fn get_palette_mask(&self, x: u16, y: u16) -> u8 {
        let y = y % SCREEN_HEIGHT as u16;
        let mut at = 0xFF;
        if y & 0x10 != 0 {
            at &= 0b1111_0000;
        } else {
            at &= 0b0000_1111;
        }

        if x & 0x10 != 0 {
            at &= 0b1100_1100;
        } else {
            at &= 0b0011_0011;
        }
        at
    }
}

impl Default for TileAttribute {
    fn default() -> TileAttribute {
        TileAttribute { bits: 0 }
    }
}

pub struct BackgroundRenderer {
    idx: Box<[[u8; TILES_PER_LINE]; SCREEN_HEIGHT]>,
    tile: Box<[[TilePattern; TILES_PER_LINE]; SCREEN_HEIGHT]>,
    attr: Box<[[TileAttribute; TILES_PER_LINE]; SCREEN_HEIGHT]>,

    background_buffer: Box<[PaletteIndex; SCREEN_BUFFER_SIZE]>,
}

impl BackgroundRenderer {
    pub fn render(&mut self, start_px: usize, stop_px: usize, reg: &PPUReg) {
        self.draw(reg, start_px, stop_px);
    }

    pub fn clear(&mut self) {
        self.background_buffer.copy_from_slice(&EMPTY_PIXEL_BUFFER);
    }

    pub fn run_cycle(&mut self, cyc: u16, sl: i16, reg: &mut PPUReg, mem: &mut PPUMemory) {
        if !reg.ppumask.rendering_enabled() {
            return;
        }
        // Match to update vram addresses
        match (cyc, sl) {
            (280...304, -1) => self.copy_vertical(reg),
            (256, -1...239) => self.increment_y(reg),
            (257, -1...239) => self.copy_horizontal(reg),
            (328, -1...239) | (336, -1...239) => self.increment_x(reg),
            (1...256, -1...239) if cyc % 8 == 0 => self.increment_x(reg),
            _ => (),
        }
        // VRAM Accesses
        match (cyc, sl, cyc % 8) {
            // Fetches for next scanline
            (320...336, -1...239, 1) => {
                self.fetch_nametable((cyc - 320) / 8, (sl + 1) % 240, reg, mem)
            }
            (320...336, -1...239, 3) => {
                self.fetch_attribute((cyc - 320) / 8, (sl + 1) % 240, reg, mem)
            }
            (320...336, -1...239, 5) => {
                self.fetch_tile_pattern((cyc - 320) / 8, (sl + 1) % 240, reg, mem)
            }

            // Fetches for this scanline
            (0...256, 0...239, 1) => self.fetch_nametable((cyc + 16) / 8, sl, reg, mem),
            (0...256, 0...239, 3) => self.fetch_attribute((cyc + 16) / 8, sl, reg, mem),
            (0...256, 0...239, 5) => self.fetch_tile_pattern((cyc + 16) / 8, sl, reg, mem),

            // The two garbage nametable fetches at the end of every scanline
            (337, -1...239, _) | (339, -1...239, _) => self.garbage_nametable_fetch(reg, mem),

            _ => (),
        }
    }

    fn copy_vertical(&self, reg: &mut PPUReg) {
        let vertical_mask = 0b_111_10_11111_00000;
        reg.v = (reg.v & !vertical_mask) | (reg.t & vertical_mask);
    }

    fn copy_horizontal(&self, reg: &mut PPUReg) {
        let horizontal_mask = 0b_000_01_00000_11111;
        reg.v = (reg.v & !horizontal_mask) | (reg.t & horizontal_mask);
    }

    fn increment_x(&self, reg: &mut PPUReg) {
        if (reg.v & 0x001F) == 31 {
            reg.v &= !0x001F; //clear coarse x
            reg.v ^= 0x0400; //Switch nametable
        } else {
            reg.v += 1; //increment coarse x
        }
    }

    fn increment_y(&self, reg: &mut PPUReg) {
        if (reg.v & 0x7000) != 0x7000 {
            reg.v += 0x1000; //Increment fine Y
        } else {
            reg.v &= !0x7000; //Clear fine Y
            let mut coarse_y = (reg.v & 0x03E0) >> 5;
            if coarse_y == 29 {
                coarse_y = 0;
                reg.v ^= 0x0800; //Switch vertical nametable
            } else if coarse_y == 31 {
                coarse_y = 0; //Clear coarse_y, but do not switch nametable
            } else {
                coarse_y += 1;
            }
            reg.v = (reg.v & !0x03E0) | (coarse_y << 5); //copy coarse_y back into V.
        }
    }

    fn fetch_nametable(&mut self, tile_x: u16, y: i16, reg: &PPUReg, mem: &mut PPUMemory) {
        let nametable_addr = 0x2000 | (reg.v & 0x0FFF);
        self.idx[y as usize][tile_x as usize] = mem.read(nametable_addr);
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn fetch_attribute(&mut self, tile_x: u16, y: i16, reg: &PPUReg, mem: &mut PPUMemory) {
        let attribute_addr = 0x23C0 | (reg.v & 0x0C00) | ((reg.v >> 4) & 0x38) | ((reg.v >> 2) & 0x07);
        self.attr[y as usize][tile_x as usize] = TileAttribute::new(mem.read(attribute_addr));
    }

    fn fetch_tile_pattern(&mut self, tile_x: u16, y: i16, reg: &PPUReg, mem: &mut PPUMemory) {
        self.tile[y as usize][tile_x as usize] =
            mem.read_tile_pattern(self.idx[y as usize][tile_x as usize],
                                  reg.scroll_y_fine(),
                                  reg.ppuctrl.background_table());
    }

    fn garbage_nametable_fetch(&mut self, reg: &PPUReg, mem: &mut PPUMemory) {
        let nametable_addr = 0x2000 | (reg.v & 0x0FFF);
        mem.read(nametable_addr);
    }

    fn draw(&mut self, reg: &PPUReg, start: usize, stop: usize) {
        let mut current_scanline = start / SCREEN_WIDTH;
        let mut last_scanline_boundary = current_scanline * SCREEN_WIDTH;
        let next_scanline = current_scanline + 1;
        let mut next_scanline_boundary = next_scanline * SCREEN_WIDTH;

        let mut current = start;
        while current < stop {
            let segment_start = current - last_scanline_boundary;
            let segment_end = cmp::min(next_scanline_boundary, stop) - last_scanline_boundary;

            self.draw_segment(reg,
                              current_scanline,
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

    fn draw_segment(&mut self,
                    reg: &PPUReg,
                    scanline: usize,
                    line_start: usize,
                    line_stop: usize,
                    start: usize,
                    stop: usize) {
        let pattern_line = &self.tile[scanline];
        let attr_line = &self.attr[scanline];
        let pixel_line = &mut self.background_buffer[line_start..line_stop];

        for (pixel, item) in pixel_line.iter_mut().enumerate().take(stop).skip(start) {
            let fine_x_scroll = reg.scroll_x_fine();
            let displayed_pixel = pixel + fine_x_scroll as usize;
            let tile_idx = displayed_pixel / 8;
            let pattern = pattern_line[tile_idx];
            let fine_x = displayed_pixel as u32 & 0x07;
            let color_id = pattern.get_color_in_pattern(fine_x);

            let attr = attr_line[tile_idx];
            let palette_id = attr.get_palette(pixel as u16 + reg.get_scroll_x() as u16,
                                              scanline as u16 + reg.get_scroll_y() as u16);

            *item = PaletteIndex {
                set: PaletteSet::Background,
                palette_id: palette_id,
                color_id: color_id,
            }
        }
    }

    pub fn buffer(&self) -> &[PaletteIndex] {
        &*self.background_buffer
    }

    #[allow(dead_code)]
    pub fn dump_background_pixels(&self) {
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let idx = y * SCREEN_WIDTH + x;
                let pix = self.background_buffer[idx];
                if pix.is_transparent() {
                    print!(" ");
                } else {
                    print!("{}", pix.color_id);
                }
            }
            println!("");
        }
        println!("");
    }

    #[allow(dead_code)]
    pub fn dump_background_palettes(&self) {
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let idx = y * SCREEN_WIDTH + x;
                let pix = self.background_buffer[idx];
                if pix.is_transparent() {
                    print!(" ");
                } else {
                    print!("{}", pix.palette_id);
                }
            }
            println!("");
        }
        println!("");
    }

    #[cfg(feature="mousepick")]
    pub fn mouse_pick(&self, reg: &PPUReg, px_x: i32, px_y: i32) {
        let scanline = px_y as usize;
        let tile = (px_x / 8) as usize;
        let tile_id = self.idx[scanline][tile];
        let attr = self.attr[scanline][tile];
        let scrolled_x = px_x as u16 + reg.get_scroll_x() as u16;
        let scrolled_y = px_y as u16 + reg.get_scroll_y() as u16;
        let palette = attr.get_palette(scrolled_x, scrolled_y);
        let palette_mask = attr.get_palette_mask(scrolled_x, scrolled_y);
        println!("{:03}/{:03}: Tile: {:03}, Attribute: {:08b} & {:08b}, Palette: {}",
                 scrolled_x / 8,
                 scrolled_y / 8,
                 tile_id,
                 attr.bits,
                 palette_mask,
                 palette);
    }
}

impl Default for BackgroundRenderer {
    fn default() -> BackgroundRenderer {
        // Work around the 32-element array limitation
        let (idx, tiles, attrs) = unsafe {
            use std::ptr;
            use std::mem;

            let mut idx: [[u8; TILES_PER_LINE]; SCREEN_HEIGHT] = mem::uninitialized();
            let mut tiles: [[TilePattern; TILES_PER_LINE]; SCREEN_HEIGHT] = mem::uninitialized();
            let mut attrs: [[TileAttribute; TILES_PER_LINE]; SCREEN_HEIGHT] = mem::uninitialized();

            for element in idx.iter_mut() {
                let idx_line: [u8; TILES_PER_LINE] = [0; TILES_PER_LINE];
                ptr::write(element, idx_line);
            }
            for element in tiles.iter_mut() {
                let tile_line: [TilePattern; TILES_PER_LINE] = [Default::default(); TILES_PER_LINE];
                ptr::write(element, tile_line);
            }
            for element in attrs.iter_mut() {
                let attr_line: [TileAttribute; TILES_PER_LINE] =
                    [Default::default(); TILES_PER_LINE];
                ptr::write(element, attr_line);
            }

            (idx, tiles, attrs)
        };

        BackgroundRenderer {
            idx: Box::new(idx),
            tile: Box::new(tiles),
            attr: Box::new(attrs),

            background_buffer: Box::new([Default::default(); SCREEN_BUFFER_SIZE]),
        }
    }
}
