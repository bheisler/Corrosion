use super::PPU;
use super::PaletteIndex;
use super::PaletteSet;
use super::TilePattern;
use memory::MemSegment;

const NAMETABLE_WIDTH: usize = 32;

pub struct BackgroundRenderer {
    tile: TilePattern,
    attr: u8,
    fetch: bool,
}

impl Default for BackgroundRenderer {
    fn default() -> BackgroundRenderer {
        BackgroundRenderer {
            tile: Default::default(),
            attr: 0,
            fetch: true,
        }
    }
}

impl PPU {
    pub fn visible_scanline_background(&mut self, pixel: u16, scanline: u16) {
        let x = pixel + self.reg.scroll_x() as u16;
        let y = scanline + self.reg.scroll_y() as u16;
        if self.background_data.fetch {
            let nametable_addr = self.get_nametable_addr(x, y);
            let tile_idx = self.ppu_mem.read(nametable_addr);

            let tile_table = self.reg.ppuctrl.background_table();
            self.background_data.tile = self.read_tile_pattern(tile_idx, y & 0x07, tile_table);

            let attribute_addr = self.get_attribute_addr(x, y);
            self.background_data.attr = self.ppu_mem.read(attribute_addr);

            self.background_data.fetch = false;
        }
    }

    pub fn get_background_pixel(&mut self, screen_x: u16, screen_y: u16) -> PaletteIndex {
        let x = screen_x + self.reg.scroll_x() as u16;
        let y = screen_y + self.reg.scroll_y() as u16;

        let color_id = self.get_color_id(x);
        let palette_id = self.get_palette_id(x, y);

        PaletteIndex {
            set: PaletteSet::Background,
            palette_id: palette_id,
            color_id: color_id,
        }
    }

    fn get_color_id(&mut self, x: u16) -> u8 {
        let pattern = self.background_data.tile;
        let fine_x = x as u32 & 0x07;
        self.background_data.fetch = fine_x == 7;
        self.get_color_in_pattern(pattern, fine_x)
    }

    fn get_nametable_addr(&self, px_x: u16, px_y: u16) -> u16 {
        let x = px_x / 8;
        let y = px_y / 8;
        let result = self.reg.ppuctrl.nametable_addr() + y * NAMETABLE_WIDTH as u16 + x;
        result
    }

    fn get_palette_id(&mut self, x: u16, y: u16) -> u8 {
        let attr = self.background_data.attr;
        self.get_palette_from_attribute(attr, x, y)
    }

    fn get_attribute_addr(&self, x: u16, y: u16) -> u16 {
        let x = x / 32;
        let y = y / 32;
        let attr_table = self.reg.ppuctrl.nametable_addr() + 0x03C0;
        attr_table + (y * 8) + x
    }

    fn get_palette_from_attribute(&self, attr: u8, x: u16, y: u16) -> u8 {
        let mut at = attr;
        if y & 0x10 != 0 {
            at >>= 4
        }
        if x & 0x10 != 0 {
            at >>= 2
        }
        at & 0x03
    }
}
