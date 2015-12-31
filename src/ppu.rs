use super::memory::MemSegment;
use mappers::Mapper;
use std::rc::Rc;
use std::cell::RefCell;

pub struct PPUMemory {
    cart: Rc<RefCell<Box<Mapper>>>,
    vram: [u8; 0x0800],
    palette: [u8; 0x20],
}

impl PPUMemory {
    pub fn new(cart: Rc<RefCell<Box<Mapper>>>) -> PPUMemory {
        PPUMemory {
            cart: cart,
            vram: [0u8; 0x0800],
            palette: [0u8; 0x20],
        }
    }
}

impl MemSegment for PPUMemory {
    fn read(&mut self, idx: u16) -> u8 {
        match idx {
            0x0000...0x1FFF => {
                let cart = self.cart.borrow_mut();
                cart.chr_read(idx)
            }
            0x2000...0x3EFF => self.vram[(idx % 0x800) as usize],
            0x3F00...0x3FFF => {
                match (idx - 0x3F00) as usize {
                    0x10 => self.palette[0x00],
                    0x14 => self.palette[0x04],
                    0x18 => self.palette[0x08],
                    0x1C => self.palette[0x0C],
                    x => self.palette[x],
                }
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
            0x2000...0x3EFF => self.vram[(idx % 0x800) as usize] = val,
            0x3F00...0x3FFF => {
                match (idx - 0x3F00) as usize {
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

#[derive(Debug, Copy, Clone, PartialEq)]
enum AddrByte {
    First,
    Second,
}

pub struct PPU {
    ppuctrl: u8,
    ppumask: u8,
    ppustat: u8,
    oamaddr: u8,
    ppuscroll: u16,
    ppuaddr: u16,

    oam: [u8; 256],

    ///A fake dynamic latch representing the capacitance of the wires in the
    ///PPU that we have to emulate.
    dyn_latch: u8,

    ///The address registers are two bytes but we can only write one at a time.
    address_latch: AddrByte,

    ppu_mem: PPUMemory,
}

impl PPU {
    pub fn new(ppu_mem: PPUMemory) -> PPU {
        PPU {
            ppuctrl: 0,
            ppumask: 0,
            ppustat: 0,
            oamaddr: 0,
            ppuscroll: 0,
            ppuaddr: 0,
            dyn_latch: 0,
            address_latch: AddrByte::First,
            oam: [0u8; 256],
            ppu_mem: ppu_mem,
        }
    }
    
    fn incr_ppuaddr(&mut self) {
        let incr_size = if (self.ppuctrl & 0b0000_0100) == 0 {
            1
        } else {
            32
        };
        self.ppuaddr = self.ppuaddr.wrapping_add(incr_size);
    }
}

fn write_addr_byte(latch: &mut AddrByte, target: &mut u16, val: u8) {
    match *latch {
        AddrByte::First =>  { *target = (*target & 0x00FF) | ((val as u16) << 8); }
        AddrByte::Second => { *target = (*target & 0xFF00) | ((val as u16) << 0); }
    }
    *latch = AddrByte::Second;
}

impl MemSegment for PPU {
    fn read(&mut self, idx: u16) -> u8 {
        match idx % 8 {
            0x0000 => self.dyn_latch,
            0x0001 => self.dyn_latch,
            0x0002 => {
                self.address_latch = AddrByte::First;
                (self.ppustat & 0b1110_0000) | (self.dyn_latch & 0b0001_1111)
            }
            0x0003 => self.dyn_latch,
            0x0004 => {
                let res = self.oam[self.oamaddr as usize];
                self.oamaddr = self.oamaddr.wrapping_add(1);
                res
            }
            0x0005 => self.dyn_latch,
            0x0006 => self.dyn_latch,
            0x0007 => {
                let res = self.ppu_mem.read(self.ppuaddr);
                self.incr_ppuaddr();
                res
            }
            x => invalid_address!(x),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        self.dyn_latch = val;
        match idx % 8 {
            0x0000 => self.ppuctrl = val,
            0x0001 => self.ppumask = val,
            0x0002 => (),
            0x0003 => self.oamaddr = val,
            0x0004 => {
                self.oam[self.oamaddr as usize] = val;
                self.oamaddr = self.oamaddr.wrapping_add(1);
            }
            0x0005 => write_addr_byte(&mut self.address_latch, &mut self.ppuscroll, val),
            0x0006 => write_addr_byte(&mut self.address_latch, &mut self.ppuaddr, val),
            0x0007 => {
                self.ppu_mem.write(self.ppuaddr, val);
                self.incr_ppuaddr();
            }
            x => invalid_address!(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory::MemSegment;
    use mappers::Mapper;
    use std::rc::Rc;
    use std::cell::RefCell;
    use ppu::AddrByte;

    fn create_test_ppu() -> PPU {
        create_test_ppu_with_rom(vec![0u8; 0x1000])
    }

    fn create_test_ppu_with_rom(chr_rom: Vec<u8>) -> PPU {
        let cart = Mapper::new(0, vec![0u8; 0x1000], chr_rom, vec![0u8; 0x1000]);
        let ppu_mem = PPUMemory::new(Rc::new(RefCell::new(cart)));
        PPU::new(ppu_mem)
    }

    fn assert_register_single_writable(idx: u16, getter: &Fn(&PPU) -> u8) {
        let mut ppu = create_test_ppu();
        ppu.write(idx, 12);
        assert_eq!(getter(&ppu), 12);
        ppu.write(idx, 125);
        assert_eq!(getter(&ppu), 125);
    }

    fn assert_register_double_writable(idx: u16, getter: &Fn(&PPU) -> u16) {
        let mut ppu = create_test_ppu();
        ppu.write(idx, 0xDE);
        assert_eq!(getter(&ppu), 0xDE00);
        ppu.write(idx, 0xAD);
        assert_eq!(getter(&ppu), 0xDEAD);
        ppu.write(idx, 0xED);
        assert_eq!(getter(&ppu), 0xDEED);
        ppu.address_latch = AddrByte::First;
        ppu.write(idx, 0xAD);
        assert_eq!(getter(&ppu), 0xADED);
    }

    fn assert_register_ignores_writes(idx: u16, getter: &Fn(&PPU) -> u8) {
        let mut ppu = create_test_ppu();
        ppu.write(idx, 12);
        assert_eq!(getter(&ppu), 0);
        ppu.write(idx, 125);
        assert_eq!(getter(&ppu), 0);
    }

    fn assert_writing_register_fills_latch(idx: u16) {
        let mut ppu = create_test_ppu();
        ppu.write(idx, 12);
        assert_eq!(ppu.dyn_latch, 12);
        ppu.write(idx, 125);
        assert_eq!(ppu.dyn_latch, 125);
    }

    fn assert_register_is_readable(idx: u16, setter: &Fn(&mut PPU, u8) -> ()) {
        let mut ppu = create_test_ppu();
        setter(&mut ppu, 12);
        assert_eq!(ppu.read(idx), 12);
        setter(&mut ppu, 125);
        assert_eq!(ppu.read(idx), 125);
    }

    fn assert_register_not_readable(idx: u16) {
        let mut ppu = create_test_ppu();
        ppu.dyn_latch = 12;
        assert_eq!(ppu.read(idx), 12);
        ppu.dyn_latch = 125;
        assert_eq!(ppu.read(idx), 125);
    }

    #[test]
    fn ppuctrl_is_write_only_register() {
        assert_register_single_writable(0x2000, &|ref ppu| ppu.ppuctrl);
        assert_writing_register_fills_latch(0x2000);
        assert_register_not_readable(0x2000);
    }

    #[test]
    fn ppu_mirrors_address() {
        assert_register_single_writable(0x2008, &|ref ppu| ppu.ppuctrl);
        assert_register_single_writable(0x2010, &|ref ppu| ppu.ppuctrl);
    }

    #[test]
    fn ppumask_is_write_only_register() {
        assert_register_single_writable(0x2001, &|ref ppu| ppu.ppumask);
        assert_writing_register_fills_latch(0x2001);
        assert_register_not_readable(0x2001);
    }

    #[test]
    fn ppustat_is_read_only_register() {
        assert_register_ignores_writes(0x2002, &|ref ppu| ppu.ppustat);
        assert_writing_register_fills_latch(0x2002);
        assert_register_is_readable(0x2002,
                                    &|ref mut ppu, val| {
                                        ppu.ppustat = val;
                                        ppu.dyn_latch = val;
                                    });
    }

    #[test]
    fn reading_ppustat_returns_part_of_dynlatch() {
        let mut ppu = create_test_ppu();
        ppu.dyn_latch = 0b0001_0101;
        ppu.ppustat = 0b1010_0101;
        assert_eq!(ppu.read(0x2002), 0b1011_0101);
    }

    #[test]
    fn reading_ppustat_clears_addr_latch() {
        let mut ppu = create_test_ppu();
        ppu.address_latch = AddrByte::Second;
        ppu.read(0x2002);
        assert_eq!(ppu.address_latch, AddrByte::First);
    }

    #[test]
    fn oamaddr_is_write_only_register() {
        assert_register_single_writable(0x2003, &|ref ppu| ppu.oamaddr);
        assert_writing_register_fills_latch(0x2003);
        assert_register_not_readable(0x2003);
    }

    #[test]
    fn ppuscroll_is_2x_write_only_register() {
        assert_register_double_writable(0x2005, &|ref ppu| ppu.ppuscroll);
        assert_writing_register_fills_latch(0x2005);
        assert_register_not_readable(0x2005);
    }

    #[test]
    fn ppuaddr_is_2x_write_only_register() {
        assert_register_double_writable(0x2006, &|ref ppu| ppu.ppuaddr);
        assert_writing_register_fills_latch(0x2006);
        assert_register_not_readable(0x2006);
    }

    #[test]
    fn reading_oamdata_uses_oamaddr_as_index_into_oam() {
        let mut ppu = create_test_ppu();
        for x in 0..255 {
            ppu.oam[x] = (255 - x) as u8;
        }
        ppu.oamaddr = 0;
        assert_eq!(ppu.read(0x2004), 255);
        ppu.oamaddr = 10;
        assert_eq!(ppu.read(0x2004), 245);
    }

    #[test]
    fn reading_oamdata_increments_oamaddr() {
        let mut ppu = create_test_ppu();
        ppu.oamaddr = 0;
        ppu.read(0x2004);
        assert_eq!(ppu.oamaddr, 1);
        ppu.oamaddr = 255;
        ppu.read(0x2004);
        assert_eq!(ppu.oamaddr, 0);
    }

    #[test]
    fn writing_oamdata_uses_oamaddr_as_index_into_oam() {
        let mut ppu = create_test_ppu();
        ppu.oamaddr = 0;
        ppu.write(0x2004, 12);
        assert_eq!(ppu.oam[0], 12);
        ppu.oamaddr = 10;
        ppu.write(0x2004, 15);
        assert_eq!(ppu.oam[10], 15);
    }

    #[test]
    fn writing_oamdata_increments_oamaddr() {
        let mut ppu = create_test_ppu();
        ppu.oamaddr = 0;
        ppu.write(0x2004, 12);
        assert_eq!(ppu.oamaddr, 1);
        ppu.oamaddr = 255;
        ppu.write(0x2004, 12);
        assert_eq!(ppu.oamaddr, 0);
    }

    #[test]
    fn ppu_can_read_chr_rom() {
        let mut chr_rom = vec![0u8; 0x2000];
        chr_rom[0x0ABC] = 12;
        chr_rom[0x0DBA] = 212;
        let mut ppu = create_test_ppu_with_rom(chr_rom);

        ppu.ppuaddr = 0x0ABC;
        assert_eq!(ppu.read(0x2007), 12);

        ppu.ppuaddr = 0x0DBA;
        assert_eq!(ppu.read(0x2007), 212);
    }

    #[test]
    fn ppu_can_read_write_vram() {
        let mut ppu = create_test_ppu();

        ppu.ppuaddr = 0x2ABC;
        ppu.write(0x2007, 12);
        ppu.ppuaddr = 0x2ABC;
        assert_eq!(ppu.read(0x2007), 12);

        ppu.ppuaddr = 0x2DBA;
        ppu.write(0x2007, 212);
        ppu.ppuaddr = 0x2DBA;
        assert_eq!(ppu.read(0x2007), 212);

        // Mirroring
        ppu.ppuaddr = 0x2EFC;
        ppu.write(0x2007, 128);
        ppu.ppuaddr = 0x3EFC;
        assert_eq!(ppu.read(0x2007), 128);
    }

    #[test]
    fn accessing_ppudata_increments_ppuaddr() {
        let mut ppu = create_test_ppu();
        ppu.ppuaddr = 0x2000;
        ppu.read(0x2007);
        assert_eq!(ppu.ppuaddr, 0x2001);
        ppu.write(0x2007, 0);
        assert_eq!(ppu.ppuaddr, 0x2002);
    }

    #[test]
    fn accessing_ppudata_increments_ppuaddr_by_32_when_ctrl_flag_is_set() {
        let mut ppu = create_test_ppu();
        ppu.ppuctrl = 0b0000_0100;
        ppu.ppuaddr = 0x2000;
        ppu.read(0x2007);
        assert_eq!(ppu.ppuaddr, 0x2020);
        ppu.write(0x2007, 0);
        assert_eq!(ppu.ppuaddr, 0x2040);
    }

    #[test]
    fn ppu_can_read_write_palette() {
        let mut ppu = create_test_ppu();

        ppu.ppuaddr = 0x3F00;
        ppu.write(0x2007, 12);
        ppu.ppuaddr = 0x3F00;
        assert_eq!(ppu.ppu_mem.palette[0], 12);

        ppu.ppuaddr = 0x3F01;
        ppu.write(0x2007, 212);
        ppu.ppuaddr = 0x3F01;
        assert_eq!(ppu.read(0x2007), 212);
    }

    #[test]
    fn test_palette_mirroring() {
        let mut ppu = create_test_ppu();

        let mirrors = [0x3F10, 0x3F14, 0x3F18, 0x3F1C];
        let targets = [0x3F00, 0x3F04, 0x3F08, 0x3F0C];
        for x in 0..4 {

            ppu.ppuaddr = targets[x];
            ppu.write(0x2007, 12);
            ppu.ppuaddr = mirrors[x];
            assert_eq!(ppu.read(0x2007), 12);

            ppu.ppuaddr = mirrors[x];
            ppu.write(0x2007, 12);
            ppu.ppuaddr = targets[x];
            assert_eq!(ppu.read(0x2007), 12);
        }
    }
}
