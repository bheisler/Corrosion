use ppu::{Color, SCREEN_BUFFER_SIZE};
use screen::Screen;
use sdl2::{Sdl, VideoSubsystem};
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl2::render::{Renderer, Texture};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

pub struct SDLScreen<'a> {
    video: VideoSubsystem,
    renderer: Renderer<'a>,
    texture: Texture,
}

const SCREEN_WIDTH: u32 = ::ppu::SCREEN_WIDTH as u32;
const SCREEN_HEIGHT: u32 = ::ppu::SCREEN_HEIGHT as u32;

impl<'a> SDLScreen<'a> {
    pub fn new(sdl_context: &Sdl) -> SDLScreen<'a> {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Corrosion", SCREEN_WIDTH, SCREEN_HEIGHT)
                                    .position_centered()
                                    .opengl()
                                    .build()
                                    .unwrap();

        let mut renderer = window.renderer().present_vsync().build().unwrap();

        let mut texture = renderer.create_texture_streaming(PixelFormatEnum::RGB24,
                                                            (SCREEN_WIDTH, SCREEN_HEIGHT))
                                  .unwrap();
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                   for y in 0..::ppu::SCREEN_HEIGHT {
                       for x in 0..::ppu::SCREEN_WIDTH {
                           let offset = y * pitch + x * 3;
                           buffer[offset + 0] = x as u8;
                           buffer[offset + 1] = y as u8;
                           buffer[offset + 2] = 0;
                       }
                   }
               })
               .unwrap();

        renderer.clear();
        renderer.copy(&texture,
                      None,
                      Some(Rect::new_unwrap(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)));
        renderer.present();

        SDLScreen {
            video: video_subsystem,
            renderer: renderer,
            texture: texture,
        }
    }
}

impl<'a> Screen for SDLScreen<'a> {
    fn draw(&mut self, buf: &[Color; SCREEN_BUFFER_SIZE]) {
        // TODO
    }
}
