use super::PPU;
use super::PaletteIndex;
use super::PaletteSet;
use super::TilePattern;
use super::SCREEN_BUFFER_SIZE;
use super::SCREEN_WIDTH;
use super::SCREEN_HEIGHT;
use memory::MemSegment;

const NAMETABLE_WIDTH: usize = 32;
const TILES_PER_LINE: usize = NAMETABLE_WIDTH + 1; //To allow for x scrolling
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

impl BackgroundRenderer {
    pub fn run(&mut self, _: u64, _: u64) {
        // TODO: Not implemented yet.
    }

    pub fn render(&mut self, _: usize, _: usize) {
        // TODO: Not implemented yet.
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

impl PPU {
    pub fn visible_scanline_background(&mut self, pixel: u16, scanline: u16) {
        let x = pixel + self.reg.scroll_x() as u16;
        let y = scanline + self.reg.scroll_y() as u16;

        let idx = y as usize * TILES_PER_LINE + (pixel as usize / 8);
        if pixel > 256 {
            return;
        }

        if x % 8 == 0 {
            let nametable_addr = self.get_nametable_addr(x, y);
            let tile_idx = self.ppu_mem.read(nametable_addr);

            let tile_table = self.reg.ppuctrl.background_table();
            self.background_data.tile[idx] = self.ppu_mem
                                                 .read_tile_pattern(tile_idx, y & 0x07, tile_table);

            let attribute_addr = self.get_attribute_addr(x, y);
            self.background_data.attr[idx] = TileAttribute::new(self.ppu_mem.read(attribute_addr));
        }
    }

    pub fn draw_background_pixel(&mut self, screen_x: u16, screen_y: u16) {
        let x = screen_x + self.reg.scroll_x() as u16;
        let y = screen_y + self.reg.scroll_y() as u16;

        let idx = y as usize * TILES_PER_LINE + (screen_x as usize / 8);

        let color_id = self.get_color_id(idx, x);
        let palette_id = self.get_palette_id(idx, x, y);

        let pixel = screen_y as usize * SCREEN_WIDTH + screen_x as usize;
        self.background_data.background_buffer[pixel] = PaletteIndex {
            set: PaletteSet::Background,
            palette_id: palette_id,
            color_id: color_id,
        }
    }

    fn get_color_id(&mut self, idx: usize, x: u16) -> u8 {
        let pattern = self.background_data.tile[idx];
        let fine_x = x as u32 & 0x07;
        pattern.get_color_in_pattern(fine_x)
    }

    pub fn get_nametable_addr(&self, px_x: u16, px_y: u16) -> u16 {
        let x = px_x / 8;
        let y = px_y / 8;
        let result = self.reg.ppuctrl.nametable_addr() + y * NAMETABLE_WIDTH as u16 + x;
        result
    }

    fn get_palette_id(&mut self, idx: usize, x: u16, y: u16) -> u8 {
        let attr = self.background_data.attr[idx];
        attr.get_palette(x, y)
    }

    fn get_attribute_addr(&self, x: u16, y: u16) -> u16 {
        let x = x / 32;
        let y = y / 32;
        let attr_table = self.reg.ppuctrl.nametable_addr() + 0x03C0;
        attr_table + (y * 8) + x
    }
}
