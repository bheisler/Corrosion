use super::PPU;
use super::Color;
use super::PaletteIndex;
use super::PaletteSet;
use ::memory::MemSegment;

const NAMETABLE_WIDTH: usize = 32;

impl PPU {
    pub fn get_background_pixel(&mut self, screen_x: u16, screen_y: u16) -> PaletteIndex {
        let x = screen_x + self.reg.scroll_x() as u16;
        let y = screen_y + self.reg.scroll_y() as u16;

        let color_id = self.get_color_id(x, y);
        let palette_id = self.get_palette_id(x, y);

        PaletteIndex {
            set: PaletteSet::Background,
            palette_id: palette_id,
            color_id: color_id,
        }
    }

    fn get_color_id(&mut self, x: u16, y: u16) -> u8 {
        let nametable_addr = self.get_nametable_addr(x, y);
        let tile_idx = self.ppu_mem.read(nametable_addr);

        let tile_table = self.reg.ppuctrl.background_table();
        let pattern = self.read_tile_pattern(tile_idx, y & 0x07, tile_table);

        self.get_color_in_pattern(pattern, x as u32 & 0x07)
    }

    fn get_nametable_addr(&self, px_x: u16, px_y: u16) -> u16 {
        let x = px_x / 8;
        let y = px_y / 8;
        let result = self.reg.ppuctrl.nametable_addr() + y * NAMETABLE_WIDTH as u16 + x;
        result
    }

    fn read_tile_pattern(&mut self, tile_id: u8, fine_y_scroll: u16, tile_table: u16) -> (u8, u8) {
        let lo_addr = self.get_tile_addr(tile_id, 0, fine_y_scroll, tile_table);
        let hi_addr = self.get_tile_addr(tile_id, 8, fine_y_scroll, tile_table);
        (self.ppu_mem.read(lo_addr), self.ppu_mem.read(hi_addr))
    }

    fn get_tile_addr(&self, tile_id: u8, plane: u8, fine_y_scroll: u16, tile_table: u16) -> u16 {
        let mut tile_addr = 0u16;
        tile_addr |= fine_y_scroll;
        tile_addr |= plane as u16; //Plane must be 0 for low or 8 for high
        tile_addr |= (tile_id as u16) << 4;
        tile_addr |= tile_table; //Table must be 0x0000 or 0x1000
        tile_addr
    }

    fn get_color_in_pattern(&self, pattern: (u8, u8), fine_x: u32) -> u8 {
        let (lo, hi) = pattern;
        let shift = 0x07 - fine_x;
        let color_id_lo = lo.wrapping_shr(shift) & 0x01;
        let color_id_hi = (hi.wrapping_shr(shift) & 0x01) << 1;
        color_id_lo | color_id_hi
    }

    fn get_palette_id(&mut self, x: u16, y: u16) -> u8 {
        let attribute_addr = self.get_attribute_addr(x, y);
        let attribute_byte = self.ppu_mem.read(attribute_addr);
        self.get_palette_from_attribute(attribute_byte, x, y)
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