use ppu::{Color, SCREEN_BUFFER_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};
use screen::Screen;
use sdl2::{Sdl, VideoSubsystem};
use sdl2::render::{Renderer, Texture};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

#[allow(dead_code)]
pub struct SDLScreen<'a> {
    video: VideoSubsystem,
    renderer: Renderer<'a>,
    texture: Texture,
}

const SCALE: usize = 3;

impl<'a> SDLScreen<'a> {
    pub fn new(sdl_context: &Sdl) -> SDLScreen<'a> {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Corrosion",
                                            (SCREEN_WIDTH * SCALE) as u32,
                                            (SCREEN_HEIGHT * SCALE) as u32)
                                    .position_centered()
                                    .opengl()
                                    .build()
                                    .unwrap();

        let mut renderer = window.renderer().present_vsync().build().unwrap();
        renderer.set_logical_size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32).unwrap();

        let texture = renderer.create_texture_streaming(PixelFormatEnum::RGB24,
                                                        SCREEN_WIDTH as u32,
                                                        SCREEN_HEIGHT as u32)
                              .unwrap();
        SDLScreen {
            video: video_subsystem,
            renderer: renderer,
            texture: texture,
        }
    }
}

// Using a hard-coded palette for now. Will add .pal file support later,
// maybe proper NTSC video decoding eventually.
#[cfg_attr(rustfmt, rustfmt_skip)]
static PALETTE: [u8; 192] = [
    84, 84, 84,       0, 30, 116,       8, 16, 144,       48, 0, 136,       68, 0, 100,       92, 0, 48,        84, 4, 0,         60, 24, 0,        32, 42, 0,        8, 58, 0,         0, 64, 0,         0, 60, 0,         0, 50, 60,        0, 0, 0,          0, 0, 0,    0, 0, 0,
    152, 150, 152,    8, 76, 196,       48, 50, 236,      92, 30, 228,      136, 20, 176,     160, 20, 100,     152, 34, 32,      120, 60, 0,       84, 90, 0,        40, 114, 0,       8, 124, 0,        0, 118, 40,       0, 102, 120,      0, 0, 0,          0, 0, 0,    0, 0, 0,
    236, 238, 236,    76, 154, 236,     120, 124, 236,    176, 98, 236,     228, 84, 236,     236, 88, 180,     236, 106, 100,    212, 136, 32,     160, 170, 0,      116, 196, 0,      76, 208, 32,      56, 204, 108,     56, 180, 204,     60, 60, 60,       0, 0, 0,    0, 0, 0,
    236, 238, 236,    168, 204, 236,    188, 188, 236,    212, 178, 236,    236, 174, 236,    236, 174, 212,    236, 180, 176,    228, 196, 144,    204, 210, 120,    180, 222, 120,    168, 226, 144,    152, 226, 180,    160, 214, 228,    160, 162, 160,    0, 0, 0,    0, 0, 0,
];

fn copy_to_texture(buf: &[Color; SCREEN_BUFFER_SIZE], buffer: &mut [u8], pitch: usize) {
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let nes_idx = y * SCREEN_WIDTH + x;
            let color = buf[nes_idx];
            let pal_idx = color.bits() as usize * 3;
            let offset = y * pitch + x * 3;
            buffer[offset] = PALETTE[pal_idx];
            buffer[offset + 1] = PALETTE[pal_idx + 1];
            buffer[offset + 2] = PALETTE[pal_idx + 2];
        }
    }
}

impl<'a> Screen for SDLScreen<'a> {
    fn draw(&mut self, buf: &[Color; SCREEN_BUFFER_SIZE]) {
        self.texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                copy_to_texture(buf, buffer, pitch);
            })
            .unwrap();

        self.renderer.copy(&self.texture,
                           None,
                           Some(Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)))
                           .unwrap();
        self.renderer.present();
    }
}
