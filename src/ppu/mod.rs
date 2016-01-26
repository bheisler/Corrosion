use memory::MemSegment;
use screen::Screen;
use cart::Cart;
use std::rc::Rc;
use std::cell::RefCell;
use std::default::Default;

mod ppu_reg;
use ppu::ppu_reg::*;

mod ppu_memory;
use ppu::ppu_memory::*;

mod sprite_rendering;
use ppu::sprite_rendering::*;

mod background_rendering;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;
pub const SCREEN_BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Color(u8);
impl Color {
    fn from_bits_truncate(val: u8) -> Color {
        Color(val & 0b0011_1111)
    }

    pub fn bits(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PaletteSet {
    Background,
    Sprite,
}

impl PaletteSet {
    fn table(&self) -> u16 {
        match *self {
            PaletteSet::Background => 0x00,
            PaletteSet::Sprite => 0x10,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PaletteIndex {
    pub set: PaletteSet,
    pub palette_id: u8,
    pub color_id: u8,
}

impl PaletteIndex {
    fn to_addr(self) -> u16 {
        let mut addr: u16 = 0x3F00;
        addr = addr | self.set.table();
        addr = addr | (self.palette_id as u16 & 0x03) << 2;
        addr = addr | self.color_id as u16 & 0x03;
        addr
    }

    fn is_transparent(&self) -> bool {
        self.color_id == 0
    }
}


#[derive(Debug, Copy, Clone)]
pub struct TilePattern {
    lo: u8,
    hi: u8,
}

impl Default for TilePattern {
    fn default() -> TilePattern {
        TilePattern{
            lo: 0,
            hi: 0,
        }
    }
}

pub struct PPU {
    reg: PPUReg,
    ppudata_read_buffer: u8,
    ppu_mem: PPUMemory,

    screen: Box<Screen>,
    screen_buffer: [Color; SCREEN_BUFFER_SIZE],

    sprite_data: SpriteRenderer,

    global_cyc: u64,
    cyc: u16,
    sl: i16,
    frame: u32,
}

#[derive(Copy, Debug, PartialEq, Clone)]
pub enum StepResult {
    NMI,
    Continue,
}

impl PPU {
    pub fn new(cart: Rc<RefCell<Cart>>, screen: Box<Screen>) -> PPU {
        PPU {
            reg: Default::default(),
            ppudata_read_buffer: 0,
            ppu_mem: PPUMemory::new(cart),
            screen_buffer: [Color::from_bits_truncate(0x00); SCREEN_BUFFER_SIZE],
            screen: screen,

            sprite_data: Default::default(),

            global_cyc: 0,
            cyc: 0,
            sl: 241,
            frame: 0,
        }
    }

    pub fn run_to(&mut self, cpu_cycle: u64) -> StepResult {
        let mut hit_nmi = false;
        while self.global_cyc < (cpu_cycle * 3) {
            self.tick_cycle();
            hit_nmi |= self.run_cycle();
        }

        if hit_nmi {
            StepResult::NMI
        } else {
            StepResult::Continue
        }
    }

    fn tick_cycle(&mut self) {
        self.global_cyc += 1;
        self.cyc += 1;
        if self.cyc == 341 {
            self.cyc = 0;
            self.sl += 1;
            if self.sl == 261 {
                self.sl = -1;
                self.frame += 1;
            }
        }
    }

    fn run_cycle(&mut self) -> bool {
        match (self.cyc, self.sl) {
            (c, -1) => self.prerender_scanline(c),
            (c, sl @ 0...239) => self.visible_scanline(c, sl as u16),
            (_, 240) => (), //Post-render idle scanline
            (1, 241) => return self.start_vblank(),
            (_, 241...260) => (), //VBlank lines
            _ => (),
        }
        false
    }

    fn prerender_scanline(&mut self, cycle: u16) {
        if cycle == 0 {
            self.reg.ppustat.remove(VBLANK);
        }
        if cycle == 339 && self.frame % 2 == 1 {
            self.tick_cycle()
        }
    }

    fn visible_scanline(&mut self, pixel: u16, scanline: u16) {
        // Nothing here yet
        self.visible_scanline_sprite(pixel, scanline);
        if pixel >= 256 {
            return;
        }
        let x = pixel as usize;
        let y = scanline as usize;
        self.screen_buffer[y * SCREEN_WIDTH + x] = self.get_pixel(pixel, scanline as u16);
    }

    fn get_pixel(&mut self, px: u16, sl: u16) -> Color {
        let (priority, sprite_pal_idx) = self.get_sprite_pixel(px);
        let background_pal_idx = self.get_background_pixel(px, sl);

        let pal_idx = match (background_pal_idx, priority, sprite_pal_idx) {
            (bck, _, spr) if spr.is_transparent() => bck,
            (bck, _, spr) if bck.is_transparent() => spr,
            (_, SpritePriority::Foreground, spr) => spr,
            (bck, SpritePriority::Background, _) => bck, 
        };
        self.read_palette(pal_idx)
    }

    fn start_vblank(&mut self) -> bool {
        let buf = &self.screen_buffer;
        self.screen.draw(buf);
        if self.frame > 0 {
            self.reg.ppustat.insert(VBLANK);
            self.reg.ppuctrl.generate_vblank_nmi()
        } else {
            false
        }
    }

    fn read_palette(&mut self, idx: PaletteIndex) -> Color {
        let bits = self.ppu_mem.read(idx.to_addr());
        Color::from_bits_truncate(bits)
    }

    fn read_tile_pattern(&mut self, tile_id: u8, fine_y_scroll: u16, tile_table: u16) -> TilePattern {
        let lo_addr = self.get_tile_addr(tile_id, 0, fine_y_scroll, tile_table);
        let hi_addr = self.get_tile_addr(tile_id, 8, fine_y_scroll, tile_table);
        TilePattern{
            lo: self.ppu_mem.read(lo_addr), 
            hi: self.ppu_mem.read(hi_addr),
        }
    }

    fn get_tile_addr(&self, tile_id: u8, plane: u8, fine_y_scroll: u16, tile_table: u16) -> u16 {
        let mut tile_addr = 0u16;
        tile_addr |= fine_y_scroll;
        tile_addr |= plane as u16; //Plane must be 0 for low or 8 for high
        tile_addr |= (tile_id as u16) << 4;
        tile_addr |= tile_table; //Table must be 0x0000 or 0x1000
        tile_addr
    }

    fn get_color_in_pattern(&self, pattern: TilePattern, fine_x: u32) -> u8 {
        let lo = pattern.lo;
        let hi = pattern.hi;
        let shift = 0x07 - fine_x;
        let color_id_lo = lo.wrapping_shr(shift) & 0x01;
        let color_id_hi = (hi.wrapping_shr(shift) & 0x01) << 1;
        color_id_lo | color_id_hi
    }

    #[cfg(feature="cputrace")]
    pub fn cycle(&self) -> u16 {
        self.cyc
    }

    #[cfg(feature="cputrace")]
    pub fn scanline(&self) -> i16 {
        self.sl
    }

    #[cfg(feature="cputrace")]
    pub fn vram_addr(&self) -> u16 {
        self.reg.ppuaddr
    }
}

impl MemSegment for PPU {
    fn read(&mut self, idx: u16) -> u8 {
        match idx % 8 {
            0x0004 => self.sprite_data.read(self.reg.oamaddr as u16),
            0x0007 => {
                let addr = self.reg.ppuaddr;
                match addr {
                    0x0000...0x3EFF => {
                        let old_buffer = self.ppudata_read_buffer;
                        self.ppudata_read_buffer = self.ppu_mem.read(addr);
                        self.reg.incr_ppuaddr();
                        old_buffer
                    }
                    0x3F00...0x3FFF => {
                        let read_result = self.ppu_mem.read(addr);
                        self.reg.incr_ppuaddr();
                        self.ppudata_read_buffer = self.ppu_mem.read_bypass_palette(addr);
                        read_result
                    }
                    x => invalid_address!(x),
                }
            }
            _ => self.reg.read(idx),
        }
    }

    fn write(&mut self, idx: u16, val: u8) {
        match idx % 8 {
            0x0004 => {
                self.sprite_data.write(self.reg.oamaddr as u16, val);
                self.reg.incr_oamaddr();
            }
            0x0007 => {
                self.ppu_mem.write(self.reg.ppuaddr, val);
                self.reg.incr_ppuaddr();
            }
            _ => self.reg.write(idx, val),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mappers::{Mapper, MapperParams};
    use std::rc::Rc;
    use std::cell::RefCell;
    use cart::Cart;
    use screen::DummyScreen;
    use ppu::ppu_reg::PPUCtrl;
    use memory::MemSegment;

    pub fn create_test_ppu() -> PPU {
        create_test_ppu_with_rom(vec![0u8; 0x1000])
    }

    pub fn create_test_ppu_with_rom(chr_rom: Vec<u8>) -> PPU {
        let mapper = Mapper::new(0, MapperParams::simple(vec![0u8; 0x1000], chr_rom));
        let cart = Cart::new(mapper);
        PPU::new(Rc::new(RefCell::new(cart)), Box::new(DummyScreen::new()))
    }

    #[test]
    fn reading_oamdata_doesnt_increment_oamaddr() {
        let mut ppu = create_test_ppu();
        ppu.reg.oamaddr = 0;
        ppu.read(0x2004);
        assert_eq!(ppu.reg.oamaddr, 0);
    }

    #[test]
    fn writing_oamdata_increments_oamaddr() {
        let mut ppu = create_test_ppu();
        ppu.reg.oamaddr = 0;
        ppu.write(0x2004, 12);
        assert_eq!(ppu.reg.oamaddr, 1);
        ppu.reg.oamaddr = 255;
        ppu.write(0x2004, 12);
        assert_eq!(ppu.reg.oamaddr, 0);
    }

    #[test]
    fn ppu_can_read_chr_rom() {
        let mut chr_rom = vec![0u8; 0x2000];
        chr_rom[0x0ABC] = 12;
        chr_rom[0x0DBA] = 212;
        let mut ppu = create_test_ppu_with_rom(chr_rom);

        ppu.reg.ppuaddr = 0x0ABC;
        ppu.read(0x2007);//Dummy Read
        assert_eq!(ppu.read(0x2007), 12);

        ppu.reg.ppuaddr = 0x0DBA;
        ppu.read(0x2007);//Dummy Read
        assert_eq!(ppu.read(0x2007), 212);
    }

    #[test]
    fn ppu_can_read_write_vram() {
        let mut ppu = create_test_ppu();

        ppu.reg.ppuaddr = 0x2ABC;
        ppu.write(0x2007, 12);
        ppu.reg.ppuaddr = 0x2ABC;
        ppu.read(0x2007);//Dummy read
        assert_eq!(ppu.read(0x2007), 12);

        ppu.reg.ppuaddr = 0x2DBA;
        ppu.write(0x2007, 212);
        ppu.reg.ppuaddr = 0x2DBA;
        ppu.read(0x2007);//Dummy Read
        assert_eq!(ppu.read(0x2007), 212);

        // Mirroring
        ppu.reg.ppuaddr = 0x2EFC;
        ppu.write(0x2007, 128);
        ppu.reg.ppuaddr = 0x3EFC;
        ppu.read(0x2007);//Dummy Read
        assert_eq!(ppu.read(0x2007), 128);
    }

    #[test]
    fn ppu_needs_no_dummy_read_for_palette_data() {
        let mut ppu = create_test_ppu();
        ppu.reg.ppuaddr = 0x3F16;
        ppu.write(0x2007, 21);
        ppu.reg.ppuaddr = 0x3F16;
        assert_eq!(ppu.read(0x2007), 21);
    }

    #[test]
    fn accessing_ppudata_increments_ppuaddr() {
        let mut ppu = create_test_ppu();
        ppu.reg.ppuaddr = 0x2000;
        ppu.read(0x2007);
        assert_eq!(ppu.reg.ppuaddr, 0x2001);
        ppu.write(0x2007, 0);
        assert_eq!(ppu.reg.ppuaddr, 0x2002);
    }

    #[test]
    fn accessing_ppudata_increments_ppuaddr_by_32_when_ctrl_flag_is_set() {
        let mut ppu = create_test_ppu();
        ppu.reg.ppuctrl = PPUCtrl::new(0b0000_0100);
        ppu.reg.ppuaddr = 0x2000;
        ppu.read(0x2007);
        assert_eq!(ppu.reg.ppuaddr, 0x2020);
        ppu.write(0x2007, 0);
        assert_eq!(ppu.reg.ppuaddr, 0x2040);
    }
}
