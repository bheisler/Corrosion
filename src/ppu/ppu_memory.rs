use memory::MemSegment;
use cart::{Cart, ScreenMode};
use std::rc::Rc;
use std::cell::RefCell;
use super::Color;
use super::PaletteIndex;
use super::TilePattern;

/// Represents the PPU's memory map.
pub struct PPUMemory {
    cart: Rc<RefCell<Cart>>,
    vram: [u8; 0x0F00],
    palette: [Color; 0x20],
}

fn get_nametable_addrs(mode: ScreenMode) -> [u16; 4] {
    match mode {
        ScreenMode::Vertical => [0x2000, 0x2400, 0x2000, 0x2400],
        ScreenMode::Horizontal => [0x2000, 0x2000, 0x2400, 0x2400],
        ScreenMode::OneScreenLow => [0x2000, 0x2000, 0x2000, 0x2000],
        ScreenMode::OneScreenHigh => [0x2400, 0x2400, 0x2400, 0x2400],
        ScreenMode::FourScreen => [0x2000, 0x2400, 0x2800, 0x2C00],
    }
}

impl PPUMemory {
    pub fn new(cart: Rc<RefCell<Cart>>) -> PPUMemory {
        PPUMemory {
            cart: cart,
            vram: [0u8; 0x0F00],
            palette: [Color::from_bits_truncate(0); 0x20],
        }
    }
}

fn get_tile_addr(tile_id: u8, plane: u8, fine_y_scroll: u16, tile_table: u16) -> u16 {
    let mut tile_addr = 0u16;
    tile_addr |= fine_y_scroll;
    tile_addr |= plane as u16; //Plane must be 0 for low or 8 for high
    tile_addr |= (tile_id as u16) << 4;
    tile_addr |= tile_table; //Table must be 0x0000 or 0x1000
    tile_addr
}

impl PPUMemory {
    pub fn read_bypass_palette(&mut self, idx: u16) -> u8 {
        let idx = self.translate_vram_address(idx);
        self.vram[idx]
    }

    fn translate_vram_address(&self, idx: u16) -> usize {
        let idx = idx & 0x0FFF;
        let nametable_num = (idx / 0x0400) as usize;
        let idx_in_nametable = idx % 0x400;
        let mode = self.cart.borrow().get_mirroring_mode();
        let translated = get_nametable_addrs(mode)[nametable_num] + idx_in_nametable;
        translated as usize % self.vram.len()
    }

    pub fn read_palette(&mut self, idx: PaletteIndex) -> Color {
        self.read_palette_mem(idx.to_addr() as usize)
    }

    fn read_palette_mem(&self, idx: usize) -> Color {
        match (idx % 0x1F) as usize {
            0x10 => self.palette[0x00],
            0x14 => self.palette[0x04],
            0x18 => self.palette[0x08],
            0x1C => self.palette[0x0C],
            x => self.palette[x],
        }
    }

    pub fn read_tile_pattern(&mut self,
                             tile_id: u8,
                             fine_y_scroll: u16,
                             tile_table: u16)
                             -> TilePattern {
        let lo_addr = get_tile_addr(tile_id, 0, fine_y_scroll, tile_table);
        let hi_addr = get_tile_addr(tile_id, 8, fine_y_scroll, tile_table);
        TilePattern {
            lo: self.read(lo_addr),
            hi: self.read(hi_addr),
        }
    }

    #[allow(dead_code)]
    pub fn dump_nametable(&mut self, idx: u16) {
        let start_idx = 0x2000 + (idx * 0x400);
        println!("Nametable {}:", idx);
        self.print_columns(start_idx..(start_idx + 0x3C0), 32)
    }

    #[allow(dead_code)]
    pub fn dump_attribute_table(&mut self, idx: u16) {
        let start_idx = 0x2000 + (idx * 0x400);
        println!("Attribute table {}:", idx);
        self.print_columns((start_idx + 0x3C0)..(start_idx + 0x400), 32);
    }
}

impl MemSegment for PPUMemory {
    fn read(&mut self, idx: u16) -> u8 {
        match idx {
            0x0000...0x1FFF => {
                let mut cart = self.cart.borrow_mut();
                cart.chr_read(idx)
            }
            0x2000...0x3EFF => self.read_bypass_palette(idx),
            0x3F00...0x3FFF => self.read_palette_mem(idx as usize).bits(),
            x => invalid_address!(x),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        match idx {
            0x0000...0x1FFF => {
                let mut cart = self.cart.borrow_mut();
                cart.chr_write(idx, val)
            }
            0x2000...0x3EFF => {
                let idx = self.translate_vram_address(idx);
                self.vram[idx] = val;
            }
            0x3F00...0x3FFF => {
                let val = Color::from_bits_truncate(val);
                match (idx & 0x001F) as usize {
                    0x10 => self.palette[0x00] = val,
                    0x14 => self.palette[0x04] = val,
                    0x18 => self.palette[0x08] = val,
                    0x1C => self.palette[0x0C] = val,
                    x => self.palette[x] = val,
                }
            }
            x => invalid_address!(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use memory::MemSegment;
    use ppu::tests::*;
    use ppu::{Color, PPU};
    use cart::ScreenMode;

    #[test]
    fn ppu_can_read_write_palette() {
        let mut ppu = create_test_ppu();

        ppu.reg.v = 0x3F00;
        ppu.write(0x2007, 12);
        ppu.reg.v = 0x3F00;
        assert_eq!(ppu.ppu_mem.palette[0], Color::from_bits_truncate(12));

        ppu.reg.v = 0x3F01;
        ppu.write(0x2007, 212);
        ppu.reg.v = 0x3F01;
        assert_eq!(ppu.read(0x2007), 212 & 0x3F);
    }

    #[test]
    fn test_palette_mirroring() {
        let mut ppu = create_test_ppu();

        let mirrors = [0x3F10, 0x3F14, 0x3F18, 0x3F1C];
        let targets = [0x3F00, 0x3F04, 0x3F08, 0x3F0C];
        for x in 0..4 {

            ppu.reg.v = targets[x];
            ppu.write(0x2007, 12);
            ppu.reg.v = mirrors[x];
            assert_eq!(ppu.read(0x2007), 12);

            ppu.reg.v = mirrors[x];
            ppu.write(0x2007, 12);
            ppu.reg.v = targets[x];
            assert_eq!(ppu.read(0x2007), 12);
        }
    }

    fn to_nametable_idx(idx: u16, tbl: u16) -> u16 {
        0x2000 + (0x0400 * tbl) + idx
    }

    fn assert_mirrored(ppu: &mut PPU, tbl1: u16, tbl2: u16) {
        for idx in 0x0000..0x0400 {
            let tbl1_idx = to_nametable_idx(idx, tbl1);
            let tbl2_idx = to_nametable_idx(idx, tbl2);

            println!("Translated: tbl1: {:04X}, tbl2: {:04X}",
                ppu.ppu_mem.translate_vram_address(tbl1_idx),
                ppu.ppu_mem.translate_vram_address(tbl2_idx),
            );

            ppu.ppu_mem.write(tbl1_idx, 0xFF);
            assert_eq!(0xFF, ppu.ppu_mem.read(tbl2_idx));

            ppu.ppu_mem.write(tbl2_idx, 0x61);
            assert_eq!(0x61, ppu.ppu_mem.read(tbl1_idx));
        }
    }

    fn assert_not_mirrored(ppu: &mut PPU, tbl1: u16, tbl2: u16) {
        for idx in 0x0000..0x0400 {
            let tbl1_idx = to_nametable_idx(idx, tbl1);
            let tbl2_idx = to_nametable_idx(idx, tbl2);

            println!("Translated: tbl1: {:04X}, tbl2: {:04X}",
                ppu.ppu_mem.translate_vram_address(tbl1_idx),
                ppu.ppu_mem.translate_vram_address(tbl2_idx),
            );

            ppu.ppu_mem.write(tbl1_idx, 0x00);
            ppu.ppu_mem.write(tbl2_idx, 0x00);

            ppu.ppu_mem.write(tbl1_idx, 0xFF);
            assert_eq!(0x00, ppu.ppu_mem.read(tbl2_idx));

            ppu.ppu_mem.write(tbl2_idx, 0x61);
            assert_eq!(0xFF, ppu.ppu_mem.read(tbl1_idx));
        }
    }

    #[test]
    fn single_screen_mirroring_mirrors_both_ways() {
        let mut ppu = create_test_ppu_with_mirroring(ScreenMode::OneScreenLow);

        assert_mirrored(&mut ppu, 0, 1);
        assert_mirrored(&mut ppu, 1, 2);
        assert_mirrored(&mut ppu, 2, 3);
    }

    #[test]
    fn four_screen_mirroring_mirrors_both_ways() {
        let mut ppu = create_test_ppu_with_mirroring(ScreenMode::FourScreen);

        assert_not_mirrored(&mut ppu, 0, 1);
        assert_not_mirrored(&mut ppu, 1, 2);
        assert_not_mirrored(&mut ppu, 2, 3);
    }

    #[test]
    fn horizontal_mirroring_mirrors_horizontally() {
        let mut ppu = create_test_ppu_with_mirroring(ScreenMode::Horizontal);

        assert_mirrored(&mut ppu, 0, 1);
        assert_mirrored(&mut ppu, 2, 3);
        assert_not_mirrored(&mut ppu, 0, 2);
        assert_not_mirrored(&mut ppu, 1, 3);
    }

    #[test]
    fn vertical_mirroring_mirrors_vertically() {
        let mut ppu = create_test_ppu_with_mirroring(ScreenMode::Vertical);

        assert_not_mirrored(&mut ppu, 0, 1);
        assert_not_mirrored(&mut ppu, 2, 3);
        assert_mirrored(&mut ppu, 0, 2);
        assert_mirrored(&mut ppu, 1, 3);
    }
}
