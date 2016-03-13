use memory::MemSegment;
use cart::Cart;
use std::rc::Rc;
use std::cell::RefCell;
use super::Color;
use super::PaletteIndex;
use super::TilePattern;

///Represents the PPU's memory map.
pub struct PPUMemory {
    cart: Rc<RefCell<Cart>>,
    vram: [u8; 0x0800],
    palette: [Color; 0x20],
}

impl PPUMemory {
    pub fn new(cart: Rc<RefCell<Cart>>) -> PPUMemory {
        PPUMemory {
            cart: cart,
            vram: [0u8; 0x0800],
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
        self.vram[(idx % 0x800) as usize]
    }

    pub fn read_palette(&mut self, idx: PaletteIndex) -> Color {
        let bits = self.read(idx.to_addr());
        Color::from_bits_truncate(bits)
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
                let cart = self.cart.borrow_mut();
                cart.chr_read(idx)
            }
            0x2000...0x3EFF => self.read_bypass_palette(idx),
            0x3F00...0x3FFF => {
                match (idx & 0x001F) as usize {
                    0x10 => self.palette[0x00],
                    0x14 => self.palette[0x04],
                    0x18 => self.palette[0x08],
                    0x1C => self.palette[0x0C],
                    x => self.palette[x],
                }
                .bits()
            }
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
                let idx = ((idx - 0x2000) % 0x800) as usize;
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

#[cfg(tests)]
mod tests {
    use memory::MemSegment;
    use super::*;
    use ppu::PPU;

    #[test]
    fn ppu_can_read_write_palette() {
        let mut ppu = create_test_ppu();

        ppu.reg.ppuaddr = 0x3F00;
        ppu.write(0x2007, 12);
        ppu.reg.ppuaddr = 0x3F00;
        assert_eq!(ppu.ppu_mem.palette[0], Color::from_bits_truncate(12));

        ppu.reg.ppuaddr = 0x3F01;
        ppu.write(0x2007, 212);
        ppu.reg.ppuaddr = 0x3F01;
        assert_eq!(ppu.read(0x2007), 212 & 0x3F);
    }

    #[test]
    fn test_palette_mirroring() {
        let mut ppu = create_test_ppu();

        let mirrors = [0x3F10, 0x3F14, 0x3F18, 0x3F1C];
        let targets = [0x3F00, 0x3F04, 0x3F08, 0x3F0C];
        for x in 0..4 {

            ppu.reg.ppuaddr = targets[x];
            ppu.write(0x2007, 12);
            ppu.reg.ppuaddr = mirrors[x];
            assert_eq!(ppu.read(0x2007), 12);

            ppu.reg.ppuaddr = mirrors[x];
            ppu.write(0x2007, 12);
            ppu.reg.ppuaddr = targets[x];
            assert_eq!(ppu.read(0x2007), 12);
        }
    }
}
