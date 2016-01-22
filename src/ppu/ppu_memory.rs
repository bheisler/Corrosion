use ::memory::MemSegment;
use cart::Cart;
use std::rc::Rc;
use std::cell::RefCell;
use super::Color;

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

impl PPUMemory {
    pub fn read_bypass_palette(&mut self, idx: u16) -> u8 {
        self.vram[(idx % 0x800) as usize]
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