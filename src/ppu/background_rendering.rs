use super::PaletteIndex;
use super::PaletteSet;
use super::TilePattern;
use super::SCREEN_BUFFER_SIZE;
use super::SCREEN_WIDTH;
use super::SCREEN_HEIGHT;
use super::ppu_reg::PPUReg;
use super::ppu_memory::PPUMemory;
use memory::MemSegment;

const NAMETABLE_WIDTH: usize = 32;
const TILES_PER_LINE: usize = 34;
const TILE_BUFFER_SIZE: usize = TILES_PER_LINE * SCREEN_HEIGHT as usize;

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
    tile: Box<[TilePattern]>,
    attr: Box<[TileAttribute]>,

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
        for pixel in start_px..stop_px {
            let (y, x) = div_rem(pixel, SCREEN_WIDTH);
            self.draw_background_pixel(reg, x as u16, y as u16);
        }
    }

    // TODO: Optimize this
    fn visible_scanline_background(&mut self,
                                   pixel: u16,
                                   scanline: u16,
                                   reg: &PPUReg,
                                   mem: &mut PPUMemory) {
        let x = pixel + reg.scroll_x() as u16;
        let y = scanline + reg.scroll_y() as u16;

        let idx = y as usize * TILES_PER_LINE + (pixel as usize / 8);
        if pixel > 256 {
            return;
        }

        if x % 8 == 0 {
            let nametable_addr = self.get_nametable_addr(reg, x, y);
            let tile_idx = mem.read(nametable_addr);

            let tile_table = reg.ppuctrl.background_table();
            self.tile[idx] = mem.read_tile_pattern(tile_idx, y & 0x07, tile_table);

            let attribute_addr = self.get_attribute_addr(reg, x, y);
            self.attr[idx] = TileAttribute::new(mem.read(attribute_addr));
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

    // TODO Optimize this.
    fn draw_background_pixel(&mut self, reg: &PPUReg, screen_x: u16, screen_y: u16) {
        let x = screen_x + reg.scroll_x() as u16;
        let y = screen_y + reg.scroll_y() as u16;

        let idx = y as usize * TILES_PER_LINE + (screen_x as usize / 8);

        let color_id = self.get_color_id(idx, x);
        let palette_id = self.get_palette_id(idx, x, y);

        let pixel = screen_y as usize * SCREEN_WIDTH + screen_x as usize;
        self.background_buffer[pixel] = PaletteIndex {
            set: PaletteSet::Background,
            palette_id: palette_id,
            color_id: color_id,
        }
    }

    fn get_color_id(&mut self, idx: usize, x: u16) -> u8 {
        let pattern = self.tile[idx];
        let fine_x = x as u32 & 0x07;
        pattern.get_color_in_pattern(fine_x)
    }

    fn get_palette_id(&mut self, idx: usize, x: u16, y: u16) -> u8 {
        let attr = self.attr[idx];
        attr.get_palette(x, y)
    }

    pub fn buffer(&self) -> &[PaletteIndex] {
        &self.background_buffer
    }
}

impl Default for BackgroundRenderer {
    fn default() -> BackgroundRenderer {
        BackgroundRenderer {
            tile: vec![Default::default(); TILE_BUFFER_SIZE].into_boxed_slice(),
            attr: vec![Default::default(); TILE_BUFFER_SIZE].into_boxed_slice(),

            background_buffer: vec![Default::default(); SCREEN_BUFFER_SIZE].into_boxed_slice(),
        }
    }
}
