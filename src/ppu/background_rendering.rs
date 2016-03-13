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

const NAMETABLE_WIDTH: usize = 32;
const TILES_PER_LINE: usize = 34;

#[derive(Debug, Copy, Clone, PartialEq)]
struct TileAttribute {
    bits: u8,
}

impl TileAttribute {
    fn new(bits: u8) -> TileAttribute {
        TileAttribute { bits: bits }
    }

    fn get_palette(&self, x: u16, y: u16) -> u8 {
        let mut at = self.bits;
        if y & 0x10 != 0 {
            at >>= 4
        }
        if x & 0x10 != 0 {
            at >>= 2
        }
        at & 0x03
    }
}

impl Default for TileAttribute {
    fn default() -> TileAttribute {
        TileAttribute { bits: 0 }
    }
}

pub struct BackgroundRenderer {
    tile: Box<[[TilePattern; TILES_PER_LINE]]>,
    attr: Box<[[TileAttribute; TILES_PER_LINE]]>,

    background_buffer: Box<[PaletteIndex]>,
}

fn div_rem(num: usize, den: usize) -> (usize, usize) {
    (num / den, num % den)
}

impl BackgroundRenderer {
    pub fn render(&mut self, start_px: usize, stop_px: usize, reg: &PPUReg, mem: &mut PPUMemory) {
        for pixel in start_px..stop_px {
            let (y, x) = div_rem(pixel, SCREEN_WIDTH);
            self.visible_scanline_background(x as u16, y as u16, reg, mem);
        }
        self.draw(start_px, stop_px);
    }

    fn visible_scanline_background(&mut self, x: u16, y: u16, reg: &PPUReg, mem: &mut PPUMemory) {
        if x > 256 {
            return;
        }
        let sl = y as usize;
        let px = x as usize / 8;

        if x % 8 == 0 {
            let nametable_addr = self.get_nametable_addr(reg, x, y);
            let tile_idx = mem.read(nametable_addr);

            let tile_table = reg.ppuctrl.background_table();
            self.tile[sl][px] = mem.read_tile_pattern(tile_idx, y & 0x07, tile_table);

            let attribute_addr = self.get_attribute_addr(reg, x, y);
            self.attr[sl][px] = TileAttribute::new(mem.read(attribute_addr));
        }
    }

    fn get_nametable_addr(&self, reg: &PPUReg, px_x: u16, px_y: u16) -> u16 {
        let x = px_x / 8;
        let y = px_y / 8;
        let result = reg.ppuctrl.nametable_addr() + y * NAMETABLE_WIDTH as u16 + x;
        result
    }

    fn get_attribute_addr(&self, reg: &PPUReg, x: u16, y: u16) -> u16 {
        let x = x / 32;
        let y = y / 32;
        let attr_table = reg.ppuctrl.nametable_addr() + 0x03C0;
        attr_table + (y * 8) + x
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

            self.draw_segment(current_scanline,
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
                    scanline: usize,
                    line_start: usize,
                    line_stop: usize,
                    start: usize,
                    stop: usize) {
        let pattern_line = &self.tile[scanline];
        let attr_line = &self.attr[scanline];
        let pixel_line = &mut self.background_buffer[line_start..line_stop];

        for pixel in start..stop {
            let tile_idx = pixel / 8;
            let pattern = pattern_line[tile_idx];
            let fine_x = pixel as u32 & 0x07;
            let color_id = pattern.get_color_in_pattern(fine_x);

            let attr = attr_line[tile_idx];
            let palette_id = attr.get_palette(pixel as u16, scanline as u16);

            pixel_line[pixel] = PaletteIndex {
                set: PaletteSet::Background,
                palette_id: palette_id,
                color_id: color_id,
            }
        }
    }

    pub fn buffer(&self) -> &[PaletteIndex] {
        &self.background_buffer
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
}

impl Default for BackgroundRenderer {
    fn default() -> BackgroundRenderer {
        // Work around the 32-element array limitation
        let (tiles, attrs) = unsafe {
            use std::ptr;
            use std::mem;

            let mut tiles: [[TilePattern; TILES_PER_LINE]; SCREEN_HEIGHT] = mem::uninitialized();
            let mut attrs: [[TileAttribute; TILES_PER_LINE]; SCREEN_HEIGHT] = mem::uninitialized();

            for element in tiles.iter_mut() {
                let tile_line: [TilePattern; TILES_PER_LINE] = [Default::default(); TILES_PER_LINE];
                ptr::write(element, tile_line);
            }
            for element in attrs.iter_mut() {
                let attr_line: [TileAttribute; TILES_PER_LINE] =
                    [Default::default(); TILES_PER_LINE];
                ptr::write(element, attr_line);
            }

            (tiles, attrs)
        };

        BackgroundRenderer {
            tile: Box::new(tiles),
            attr: Box::new(attrs),

            background_buffer: vec![Default::default(); SCREEN_BUFFER_SIZE].into_boxed_slice(),
        }
    }
}
