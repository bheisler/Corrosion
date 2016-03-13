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

fn get_nametable_addr(reg: &PPUReg, tile_x: u16, tile_y: u16) -> u16 {
    let mut addr = 0x2000;
    addr |= ((tile_x + reg.scroll_x_coarse()) & 0b0001_1111) << 0;
    addr |= ((tile_y + reg.scroll_y_coarse()) & 0b0001_1111) << 5;
    addr |= reg.ppuctrl.nametable_num() << 10;
    addr
}

fn get_attribute_addr(reg: &PPUReg, tile_x: u16, tile_y: u16) -> u16 {
    let mut addr = 0x23C0;
    addr |= ((tile_x + reg.scroll_x_coarse()) & 0b0001_1100) >> 2 << 0;
    addr |= ((tile_y + reg.scroll_y_coarse()) & 0b0001_1100) >> 2 << 3;
    addr |= reg.ppuctrl.nametable_num() << 10;
    addr
}

impl BackgroundRenderer {
    pub fn render(&mut self, start_px: usize, stop_px: usize, reg: &PPUReg, mem: &mut PPUMemory) {
        self.evaluate(reg, mem, start_px, stop_px);
        self.draw(start_px, stop_px);
    }

    fn evaluate(&mut self, reg: &PPUReg, mem: &mut PPUMemory, start: usize, stop: usize) {
        let mut current_scanline = start / SCREEN_WIDTH;
        let mut last_scanline_boundary = current_scanline * SCREEN_WIDTH;
        let mut next_scanline_boundary = last_scanline_boundary + SCREEN_WIDTH;

        let mut current = start;
        while current < stop {
            let segment_start = current - last_scanline_boundary;
            let segment_end = cmp::min(next_scanline_boundary, stop) - last_scanline_boundary;

            self.evaluate_segment(reg, mem, current_scanline, segment_start, segment_end);
            current_scanline += 1;
            last_scanline_boundary = next_scanline_boundary;
            current = next_scanline_boundary;
            next_scanline_boundary += SCREEN_WIDTH;
        }
    }

    fn evaluate_segment(&mut self,
                        reg: &PPUReg,
                        mem: &mut PPUMemory,
                        scanline: usize,
                        start: usize,
                        stop: usize) {
        let tile_line = &mut self.tile[scanline];
        let attr_line = &mut self.attr[scanline];

        let tile_start = start / 8;
        let tile_stop = (stop + 8 - 1) / 8;

        let displayed_scanline = scanline as u16 + reg.scroll_y();

        for x in tile_start..tile_stop {
            let tile_x = x as u16;
            let tile_y = displayed_scanline / 8;

            let nametable_addr = get_nametable_addr(reg, tile_x, tile_y);
            let tile_table = reg.ppuctrl.background_table();
            let tile_idx = mem.read(nametable_addr);
            tile_line[x] = mem.read_tile_pattern(tile_idx, displayed_scanline & 0x07, tile_table);

            let attr_addr = get_attribute_addr(reg, tile_x, tile_y);
            attr_line[x] = TileAttribute::new(mem.read(attr_addr));
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
