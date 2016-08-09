use memory::MemSegment;
use screen::Screen;
use cart::Cart;
use std::rc::Rc;
use std::cell::UnsafeCell;
use std::default::Default;
use std::cmp;

mod ppu_reg;
use ppu::ppu_reg::*;

mod ppu_memory;
use ppu::ppu_memory::*;

mod sprite_rendering;
use ppu::sprite_rendering::*;

mod background_rendering;
use ppu::background_rendering::*;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;
pub const SCREEN_BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

const CYCLES_PER_SCANLINE: u64 = 341;
const SCANLINES_PER_FRAME: u64 = 262;
const CYCLES_PER_FRAME: u64 = CYCLES_PER_SCANLINE * SCANLINES_PER_FRAME;

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
    fn table(&self) -> u8 {
        match *self {
            PaletteSet::Background => 0x00,
            PaletteSet::Sprite => 0x10,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct PaletteIndex {
    addr: u8,
}

const TRANSPARENT: PaletteIndex = PaletteIndex{ addr: 0x00 };

impl PaletteIndex {
    pub fn from_packed(addr: u8) -> PaletteIndex {
        PaletteIndex{ addr: addr }
    }

    pub fn from_unpacked( set: PaletteSet,
        palette_id: u8,
        color_id: u8 ) -> PaletteIndex {
        if color_id == 0 {
            return PaletteIndex{ addr: 0 }
        }
        let mut addr: u8 = 0x00;
        addr |= set.table();
        addr |= (palette_id & 0x03) << 2;
        addr |= color_id & 0x03;
        PaletteIndex{ addr: addr }
    }

    #[cfg(not(feature="vectorize"))]
    fn to_index(self) -> usize {
        self.addr as usize
    }

    fn is_transparent(&self) -> bool {
        self.addr == 0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TilePattern {
    lo: u8,
    hi: u8,
}

pub const NO_TILE: TilePattern = TilePattern { lo: 0, hi : 0 };

impl Default for TilePattern {
    fn default() -> TilePattern {
        NO_TILE
    }
}

impl TilePattern {
    fn get_color_in_pattern(&self, fine_x: u32) -> u8 {
        let lo = self.lo;
        let hi = self.hi;
        let shift = 0x07 - fine_x;
        let color_id_lo = lo.wrapping_shr(shift) & 0x01;
        let color_id_hi = (hi.wrapping_shr(shift) & 0x01) << 1;
        color_id_lo | color_id_hi
    }
}

pub struct PPU {
    reg: PPUReg,
    ppudata_read_buffer: u8,
    ppu_mem: PPUMemory,

    screen: Box<Screen>,
    palette_buffer: [PaletteIndex; SCREEN_BUFFER_SIZE],
    screen_buffer: [Color; SCREEN_BUFFER_SIZE],

    sprite_data: SpriteRenderer,
    background_data: BackgroundRenderer,

    global_cyc: u64,
    cyc: u16,
    sl: i16,
    frame: u32,

    next_vblank_ppu_cyc: u64,
    next_vblank_cpu_cyc: u64,
}

#[derive(Copy, Debug, PartialEq, Clone)]
pub enum StepResult {
    NMI,
    Continue,
}

fn div_rem(num: u64, den: u64) -> (u64, u64) {
    (num / den, num % den)
}

fn ppu_to_cpu_cyc(ppu_cyc: u64) -> u64 {
    let (div, rem) = div_rem(ppu_cyc, 3);
    if rem == 0 {
        div
    } else {
        div + 1
    }
}

fn cpu_to_ppu_cyc(cpu_cyc: u64) -> u64 {
    cpu_cyc * 3
}

fn cyc_to_px(ppu_cyc: u64) -> usize {
    let mut pixel: usize = 0;
    let mut rem = ppu_cyc;

    rem += 241 * CYCLES_PER_SCANLINE;//Skip to the position at power-on.

    let (frames, rem_t) = div_rem(rem, CYCLES_PER_FRAME);
    rem = rem_t;
    pixel += frames as usize * SCREEN_BUFFER_SIZE;

    rem = rem.saturating_sub(CYCLES_PER_SCANLINE);//Skip the pre-render scanline.
    rem = cmp::min(rem, SCREEN_HEIGHT as u64 * CYCLES_PER_SCANLINE);//Cut off the VBLANK scanlines.

    let (scanlines, rem_t) = div_rem(rem, CYCLES_PER_SCANLINE);
    rem = rem_t;
    pixel += scanlines as usize * SCREEN_WIDTH;

    rem = rem.saturating_sub(1);//Skip idle cycle
    rem = cmp::min(rem, SCREEN_WIDTH as u64);//Cut off HBLANK

    pixel += rem as usize;
    pixel
}

impl PPU {
    pub fn new(cart: Rc<UnsafeCell<Cart>>, screen: Box<Screen>) -> PPU {
        PPU {
            reg: Default::default(),
            ppudata_read_buffer: 0,
            ppu_mem: PPUMemory::new(cart),

            palette_buffer: [TRANSPARENT; SCREEN_BUFFER_SIZE],
            screen_buffer: [Color::from_bits_truncate(0x00); SCREEN_BUFFER_SIZE],
            screen: screen,

            sprite_data: Default::default(),
            background_data: Default::default(),

            global_cyc: 0,
            cyc: 0,
            sl: 241,
            frame: 0,

            next_vblank_ppu_cyc: 1,
            next_vblank_cpu_cyc: ppu_to_cpu_cyc(1),
        }
    }

    pub fn run_to(&mut self, cpu_cycle: u64) -> StepResult {
        let start = self.global_cyc;
        let stop = cpu_to_ppu_cyc(cpu_cycle);

        let start_px = cyc_to_px(start);
        let delta_px = cyc_to_px(stop) - start_px;
        let start_px = start_px % SCREEN_BUFFER_SIZE;
        let stop_px = start_px + delta_px;

        let rendering_enabled = self.reg.ppumask.rendering_enabled();

        let mut hit_nmi = false;
        while self.global_cyc < stop {
            self.tick_cycle();
            self.run_cycle(rendering_enabled, &mut hit_nmi);
        }

        if self.reg.ppumask.contains( S_BCK ) {
            self.background_data.render(&mut self.palette_buffer, start_px, stop_px, &self.reg);
        }
        else {
            let slice = &mut self.palette_buffer[start_px..stop_px];
            for dest in slice.iter_mut() {
                *dest = TRANSPARENT;
            }
        }
        if self.reg.ppumask.contains( S_SPR ) {
            self.sprite_data.render(&mut self.palette_buffer, &mut self.reg, start_px, stop_px);
        }

        self.colorize(start_px, stop_px);

        if hit_nmi {
            StepResult::NMI
        } else {
            StepResult::Continue
        }
    }

    ///Returns the CPU cycle number representing the next time the CPU should run the PPU.
    ///When the CPU cycle reaches this number, the CPU must run the PPU.
    pub fn requested_run_cycle(&self) -> u64 {
        self.next_vblank_cpu_cyc
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

    fn run_cycle(&mut self, rendering_enabled: bool, hit_nmi: &mut bool) {
        if let -1...239 = self.sl {
            if rendering_enabled {
                self.background_data.run_cycle(self.cyc, self.sl, &mut self.reg, &mut self.ppu_mem);
            }
        }
        match (self.cyc, self.sl) {
            (_, -1) => self.prerender_scanline(),

            //Visible scanlines
            (0, 0...239) => self.sprite_data.sprite_eval(self.sl as u16, &self.reg, &mut self.ppu_mem),
            (_, 0...239) => (),

            (_, 240) => (), //Post-render idle scanline
            (1, 241) => self.start_vblank( hit_nmi ),
            (_, 241...260) => (), //VBlank lines
            _ => (),
        }
    }

    fn prerender_scanline(&mut self) {
        if self.cyc == 1 {
            self.reg.ppustat.remove(VBLANK | SPRITE_0 | SPRITE_OVERFLOW);
        }
        if self.cyc == 339 && self.frame % 2 == 1 {
            self.tick_cycle()
        }
    }

    fn start_vblank(&mut self, hit_nmi: &mut bool) {
        self.next_vblank_ppu_cyc += CYCLES_PER_FRAME;
        self.next_vblank_cpu_cyc = ppu_to_cpu_cyc(self.next_vblank_ppu_cyc);

        let buf = &self.screen_buffer;
        self.screen.draw(buf);

        if self.frame > 0 {
            self.reg.ppustat.insert(VBLANK);
            *hit_nmi |= self.reg.ppuctrl.generate_vblank_nmi();
        }
    }

    #[cfg(feature="vectorize")]
    fn colorize(&mut self, start:usize, stop: usize) {
        use std::mem;
        use std::cmp;
        use simd::u8x16;
        use simd::x86::ssse3::Ssse3U8x16;

        let (background_pal, sprite_pal) = self.ppu_mem.get_palettes();
        let index_bytes : &[u8; SCREEN_BUFFER_SIZE] = unsafe{ mem::transmute( &self.palette_buffer ) };
        let color_bytes : &mut [u8; SCREEN_BUFFER_SIZE] = unsafe{ mem::transmute( &mut self.screen_buffer ) };

        let mut start = start;

        while start < stop {
            start = cmp::min(start, SCREEN_BUFFER_SIZE - 16);
            let palette_idx = u8x16::load( index_bytes, start );

            let table: u8x16 = palette_idx >> 4;
            let use_sprite_table = table.ne(u8x16::splat(0));
            let color_id = palette_idx & u8x16::splat(0b0000_1111);

            let background_shuf = background_pal.shuffle_bytes(color_id);
            let sprite_shuf = sprite_pal.shuffle_bytes(color_id);

            let final_color = use_sprite_table.select( sprite_shuf, background_shuf);
            final_color.store(&mut *color_bytes, start);
            start += 16;
        }
    }

    #[cfg(not(feature="vectorize"))]
    fn colorize(&mut self, start: usize, stop: usize ) {
        let color_slice = self.screen_buffer[start...stop];
        let index_slice = self.palette_buffer[start...stop];

        for (src, dest) in color_slice.iter().zip_with(index_slice.iter_mut()) {
            *dest = self.ppu_mem.read_palette(src);
        }
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
        self.reg.v
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }

    #[cfg(feature="mousepick")]
    pub fn mouse_pick(&self, px_x: i32, px_y: i32) {
        self.background_data.mouse_pick(&self.reg, px_x, px_y);
        self.sprite_data.mouse_pick(px_x, px_y);
    }

    pub fn rendering_enabled(&self) -> bool {
        self.reg.ppumask.rendering_enabled()
    }
}

impl MemSegment for PPU {
    fn read(&mut self, idx: u16) -> u8 {
        match idx % 8 {
            0x0004 => self.sprite_data.read(self.reg.oamaddr as u16),
            0x0007 => {
                let addr = self.reg.v;
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
                self.ppu_mem.write(self.reg.v, val);
                self.reg.incr_ppuaddr();
            }
            _ => self.reg.write(idx, val),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mappers::create_test_mapper;
    use std::rc::Rc;
    use std::cell::UnsafeCell;
    use cart::{Cart, ScreenMode};
    use screen::DummyScreen;
    use ppu::ppu_reg::PPUCtrl;
    use memory::MemSegment;

    pub fn create_test_ppu() -> PPU {
        create_test_ppu_with_rom(vec![0u8; 0x1000])
    }

    pub fn create_test_ppu_with_rom(chr_rom: Vec<u8>) -> PPU {
        let mapper = create_test_mapper(vec![0u8; 0x1000], chr_rom, ScreenMode::FourScreen);
        let cart = Cart::new(mapper);
        PPU::new(Rc::new(UnsafeCell::new(cart)), Box::new(DummyScreen::default()))
    }

    pub fn create_test_ppu_with_mirroring(mode: ScreenMode) -> PPU {
        let mapper = create_test_mapper(vec![0u8; 0x1000], vec![0u8; 0x1000], mode);
        let cart = Cart::new(mapper);
        PPU::new(Rc::new(UnsafeCell::new(cart)), Box::new(DummyScreen::default()))
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

        ppu.reg.v = 0x0ABC;
        ppu.read(0x2007);//Dummy Read
        assert_eq!(ppu.read(0x2007), 12);

        ppu.reg.v = 0x0DBA;
        ppu.read(0x2007);//Dummy Read
        assert_eq!(ppu.read(0x2007), 212);
    }

    #[test]
    fn ppu_can_read_write_vram() {
        let mut ppu = create_test_ppu();

        ppu.reg.v = 0x2ABC;
        ppu.write(0x2007, 12);
        ppu.reg.v = 0x2ABC;
        ppu.read(0x2007);//Dummy read
        assert_eq!(ppu.read(0x2007), 12);

        ppu.reg.v = 0x2DBA;
        ppu.write(0x2007, 212);
        ppu.reg.v = 0x2DBA;
        ppu.read(0x2007);//Dummy Read
        assert_eq!(ppu.read(0x2007), 212);

        // Mirroring
        ppu.reg.v = 0x2EFC;
        ppu.write(0x2007, 128);
        ppu.reg.v = 0x3EFC;
        ppu.read(0x2007);//Dummy Read
        assert_eq!(ppu.read(0x2007), 128);
    }

    #[test]
    fn ppu_needs_no_dummy_read_for_palette_data() {
        let mut ppu = create_test_ppu();
        ppu.reg.v = 0x3F16;
        ppu.write(0x2007, 21);
        ppu.reg.v = 0x3F16;
        assert_eq!(ppu.read(0x2007), 21);
    }

    #[test]
    fn accessing_ppudata_increments_ppuaddr() {
        let mut ppu = create_test_ppu();
        ppu.reg.v = 0x2000;
        ppu.read(0x2007);
        assert_eq!(ppu.reg.v, 0x2001);
        ppu.write(0x2007, 0);
        assert_eq!(ppu.reg.v, 0x2002);
    }

    #[test]
    fn accessing_ppudata_increments_ppuaddr_by_32_when_ctrl_flag_is_set() {
        let mut ppu = create_test_ppu();
        ppu.reg.ppuctrl = PPUCtrl::new(0b0000_0100);
        ppu.reg.v = 0x2000;
        ppu.read(0x2007);
        assert_eq!(ppu.reg.v, 0x2020);
        ppu.write(0x2007, 0);
        assert_eq!(ppu.reg.v, 0x2040);
    }
}
