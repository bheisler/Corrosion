
use super::PaletteIndex;
use super::SCREEN_BUFFER_SIZE;
use super::SCREEN_HEIGHT;
use super::SCREEN_WIDTH;
use super::TilePattern;
use super::ppu_memory::PPUMemory;
use super::ppu_reg::PPUReg;
use memory::MemSegment;
use std::cmp;

const TILES_PER_LINE: usize = 34;

#[derive(Debug, Copy, Clone, PartialEq)]
struct TileAttribute {
    bits: u8,
}

impl TileAttribute {
    fn new(bits: u8) -> TileAttribute {
        TileAttribute { bits: bits }
    }

    fn get_palette(&self, x: u16, y: u16) -> u8 {
        let y = y % SCREEN_HEIGHT as u16;
        let mut at = self.bits;
        if y & 0x10 != 0 {
            at >>= 4
        }
        if x & 0x10 != 0 {
            at >>= 2
        }
        at & 0x03
    }

    #[cfg(feature = "debug_features")]
    fn get_palette_mask(&self, x: u16, y: u16) -> u8 {
        let y = y % SCREEN_HEIGHT as u16;
        let mut at = 0xFF;
        if y & 0x10 != 0 {
            at &= 0b1111_0000;
        } else {
            at &= 0b0000_1111;
        }

        if x & 0x10 != 0 {
            at &= 0b1100_1100;
        } else {
            at &= 0b0011_0011;
        }
        at
    }
}

impl Default for TileAttribute {
    fn default() -> TileAttribute {
        TileAttribute { bits: 0 }
    }
}

pub struct BackgroundRenderer {
    idx: Box<[[u8; TILES_PER_LINE]; SCREEN_HEIGHT]>,
    tile: Box<[[TilePattern; TILES_PER_LINE]; SCREEN_HEIGHT]>,
    attr: Box<[[u8; TILES_PER_LINE]; SCREEN_HEIGHT]>,
}

fn draw_segment(
    pattern_line: &[TilePattern; TILES_PER_LINE],
    attr_line: &[u8; TILES_PER_LINE],
    pixel_line: &mut [PaletteIndex],
    fine_x_scroll: usize,
    start: usize,
    stop: usize,
) {
    for (pixel, item) in pixel_line.iter_mut().enumerate().take(stop).skip(start) {
        let displayed_pixel = pixel + fine_x_scroll;
        render_single_pixel(pattern_line, attr_line, displayed_pixel, item);
    }
}

fn render_single_pixel(
    pattern_line: &[TilePattern],
    attr_line: &[u8],
    displayed_pixel: usize,
    item: &mut PaletteIndex,
) {
    let tile_idx = displayed_pixel / 8;
    let pattern = pattern_line[tile_idx];
    let fine_x = displayed_pixel as u32 & 0x07;
    let color_id = pattern.get_color_in_pattern(fine_x);

    let palette_id = attr_line[tile_idx];

    *item = PaletteIndex::from_packed(color_id | palette_id);
}

impl BackgroundRenderer {
    pub fn render(
        &mut self,
        buffer: &mut [PaletteIndex; SCREEN_BUFFER_SIZE],
        start: usize,
        stop: usize,
        reg: &PPUReg,
    ) {
        let mut current_scanline = start / SCREEN_WIDTH;
        let mut last_scanline_boundary = current_scanline * SCREEN_WIDTH;
        let next_scanline = current_scanline + 1;
        let mut next_scanline_boundary = next_scanline * SCREEN_WIDTH;

        let mut current = start;
        let fine_x_scroll = reg.scroll_x_fine() as usize;
        while current < stop {
            let segment_start = current - last_scanline_boundary;
            let segment_end = cmp::min(next_scanline_boundary, stop) - last_scanline_boundary;

            let pattern_line = &self.tile[current_scanline];
            let attr_line = &self.attr[current_scanline];
            let pixel_line = &mut buffer[last_scanline_boundary..next_scanline_boundary];

            draw_segment(
                pattern_line,
                attr_line,
                pixel_line,
                fine_x_scroll,
                segment_start,
                segment_end,
            );
            current_scanline += 1;
            last_scanline_boundary = next_scanline_boundary;
            current = next_scanline_boundary;
            next_scanline_boundary += SCREEN_WIDTH;
        }
    }

    pub fn run_cycle(&mut self, cyc: u16, sl: i16, reg: &mut PPUReg, mem: &mut PPUMemory) {
        // Match to update vram addresses
        self.update_vram_address(cyc, sl, reg);
        // VRAM Accesses
        self.read_data(cyc, sl, reg, mem);
    }

    fn update_vram_address(&self, cyc: u16, sl: i16, reg: &mut PPUReg) {
        if sl < 240 {
            match cyc {
                280 if sl == -1 => self.copy_vertical(reg),
                256 => self.increment_y(reg),
                257 => self.copy_horizontal(reg),
                8 |
                16 |
                24 |
                32 |
                40 |
                48 |
                56 |
                64 |
                72 |
                80 |
                88 |
                96 |
                104 |
                112 |
                120 |
                128 |
                136 |
                144 |
                152 |
                160 |
                168 |
                176 |
                184 |
                192 |
                200 |
                208 |
                216 |
                224 |
                232 |
                240 |
                248 |
                328 |
                336 => self.increment_x(reg),
                _ => (),
            }
        }
    }

    fn read_data(&mut self, cyc: u16, sl: i16, reg: &mut PPUReg, mem: &mut PPUMemory) {
        if sl == -1 {
            match cyc {
                // Fetches for next scanline
                321 | 329 => self.fetch_nametable((cyc - 320) / 8, (sl + 1) % 240, reg, mem),
                323 | 331 => self.fetch_attribute((cyc - 320) / 8, (sl + 1) % 240, reg, mem),
                325 | 333 => self.fetch_tile_pattern((cyc - 320) / 8, (sl + 1) % 240, reg, mem),

                // The two garbage nametable fetches at the end of every scanline
                337 | 339 => self.garbage_nametable_fetch(reg, mem),
                _ => (),
            }
        } else if sl < 240 {
            match cyc {
                // Normal fetches
                1 |
                9 |
                17 |
                25 |
                33 |
                41 |
                49 |
                57 |
                65 |
                73 |
                81 |
                89 |
                97 |
                105 |
                113 |
                121 |
                129 |
                137 |
                145 |
                153 |
                161 |
                169 |
                177 |
                185 |
                193 |
                201 |
                209 |
                217 |
                225 |
                233 |
                241 |
                249 => self.fetch_nametable((cyc + 16) / 8, sl, reg, mem),
                3 |
                11 |
                19 |
                27 |
                35 |
                43 |
                51 |
                59 |
                67 |
                75 |
                83 |
                91 |
                99 |
                107 |
                115 |
                123 |
                131 |
                139 |
                147 |
                155 |
                163 |
                171 |
                179 |
                187 |
                195 |
                203 |
                211 |
                219 |
                227 |
                235 |
                243 |
                251 => self.fetch_attribute((cyc + 16) / 8, sl, reg, mem),
                5 |
                13 |
                21 |
                29 |
                37 |
                45 |
                53 |
                61 |
                69 |
                77 |
                85 |
                93 |
                101 |
                109 |
                117 |
                125 |
                133 |
                141 |
                149 |
                157 |
                165 |
                173 |
                181 |
                189 |
                197 |
                205 |
                213 |
                221 |
                229 |
                237 |
                245 |
                253 => self.fetch_tile_pattern((cyc + 16) / 8, sl, reg, mem),

                // Fetches for next scanline
                321 | 329 => self.fetch_nametable((cyc - 320) / 8, (sl + 1) % 240, reg, mem),
                323 | 331 => self.fetch_attribute((cyc - 320) / 8, (sl + 1) % 240, reg, mem),
                325 | 333 => self.fetch_tile_pattern((cyc - 320) / 8, (sl + 1) % 240, reg, mem),

                // The two garbage nametable fetches at the end of every scanline
                337 | 339 => self.garbage_nametable_fetch(reg, mem),
                _ => (),
            }
        }
    }

    fn copy_vertical(&self, reg: &mut PPUReg) {
        let vertical_mask = 0b_111_10_11111_00000;
        reg.v = (reg.v & !vertical_mask) | (reg.t & vertical_mask);
    }

    fn copy_horizontal(&self, reg: &mut PPUReg) {
        let horizontal_mask = 0b_000_01_00000_11111;
        reg.v = (reg.v & !horizontal_mask) | (reg.t & horizontal_mask);
    }

    fn increment_x(&self, reg: &mut PPUReg) {
        if (reg.v & 0x001F) == 31 {
            reg.v &= !0x001F; // clear coarse x
            reg.v ^= 0x0400; // Switch nametable
        } else {
            reg.v += 1; // increment coarse x
        }
    }

    fn increment_y(&self, reg: &mut PPUReg) {
        if (reg.v & 0x7000) != 0x7000 {
            reg.v += 0x1000; // Increment fine Y
        } else {
            reg.v &= !0x7000; // Clear fine Y
            let mut coarse_y = (reg.v & 0x03E0) >> 5;
            if coarse_y == 29 {
                coarse_y = 0;
                reg.v ^= 0x0800; // Switch vertical nametable
            } else if coarse_y == 31 {
                coarse_y = 0; // Clear coarse_y, but do not switch nametable
            } else {
                coarse_y += 1;
            }
            reg.v = (reg.v & !0x03E0) | (coarse_y << 5); // copy coarse_y back into V.
        }
    }

    fn fetch_nametable(&mut self, tile_x: u16, y: i16, reg: &PPUReg, mem: &mut PPUMemory) {
        let nametable_addr = 0x2000 | (reg.v & 0x0FFF);
        self.idx[y as usize][tile_x as usize] = mem.read(nametable_addr);
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn fetch_attribute(&mut self, tile_x: u16, y: i16, reg: &PPUReg, mem: &mut PPUMemory) {
        let addr = 0x23C0 | (reg.v & 0x0C00) | ((reg.v >> 4) & 0x38) | ((reg.v >> 2) & 0x07);
        let tile_attribute = TileAttribute::new(mem.read(addr));
        let palette_id = tile_attribute.get_palette(
            tile_x * 8 + reg.get_scroll_x(),
            y as u16 + reg.get_scroll_y());
        self.attr[y as usize][tile_x as usize] = palette_id << 2;
    }

    fn fetch_tile_pattern(&mut self, tile_x: u16, y: i16, reg: &PPUReg, mem: &mut PPUMemory) {
        self.tile[y as usize][tile_x as usize] = mem.read_tile_pattern(
            self.idx[y as usize][tile_x as usize],
            reg.scroll_y_fine(),
            reg.ppuctrl.background_table(),
        );
    }

    fn garbage_nametable_fetch(&mut self, reg: &PPUReg, mem: &mut PPUMemory) {
        let nametable_addr = 0x2000 | (reg.v & 0x0FFF);
        mem.read(nametable_addr);
    }

    #[cfg(feature = "debug_features")]
    pub fn mouse_pick(&self, reg: &PPUReg, px_x: i32, px_y: i32) {
        let scanline = px_y as usize;
        let tile = (px_x / 8) as usize;
        let tile_id = self.idx[scanline][tile];
        let attr = TileAttribute::new(self.attr[scanline][tile]);
        let scrolled_x = px_x as u16 + reg.get_scroll_x() as u16;
        let scrolled_y = px_y as u16 + reg.get_scroll_y() as u16;
        let palette = attr.get_palette(scrolled_x, scrolled_y);
        let palette_mask = attr.get_palette_mask(scrolled_x, scrolled_y);
        println!(
            "{:03}/{:03}: Tile: {:03}, Attribute: {:08b} & {:08b}, Palette: {}",
            scrolled_x / 8,
            scrolled_y / 8,
            tile_id,
            attr.bits,
            palette_mask,
            palette
        );
    }
}

impl Default for BackgroundRenderer {
    fn default() -> BackgroundRenderer {
        // Work around the 32-element array limitation
        let (idx, tiles, attrs) = unsafe {
            use std::ptr;
            use std::mem;

            let mut idx: [[u8; TILES_PER_LINE]; SCREEN_HEIGHT] = mem::uninitialized();
            let mut tiles: [[TilePattern; TILES_PER_LINE]; SCREEN_HEIGHT] = mem::uninitialized();
            let mut attrs: [[u8; TILES_PER_LINE]; SCREEN_HEIGHT] = mem::uninitialized();

            for element in idx.iter_mut() {
                let idx_line: [u8; TILES_PER_LINE] = [0; TILES_PER_LINE];
                ptr::write(element, idx_line);
            }
            for element in tiles.iter_mut() {
                let tile_line: [TilePattern; TILES_PER_LINE] = [Default::default(); TILES_PER_LINE];
                ptr::write(element, tile_line);
            }
            for element in attrs.iter_mut() {
                let attr_line: [u8; TILES_PER_LINE] = [0; TILES_PER_LINE];
                ptr::write(element, attr_line);
            }

            (idx, tiles, attrs)
        };

        BackgroundRenderer {
            idx: Box::new(idx),
            tile: Box::new(tiles),
            attr: Box::new(attrs),
        }
    }
}
