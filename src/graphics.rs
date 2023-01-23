use crate::ascii_font::FONTS;
use crate::BG_COLOR;
use rusmikan::{FrameBuffer, FrameBufferConfig};

#[derive(Copy, Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub static mut GRAPHIC: Option<Graphic> = None;

pub struct Graphic {
    fb_config: FrameBufferConfig,
    pixel_writer: unsafe fn(&mut FrameBuffer, usize, Rgb),
}

impl Graphic {
    pub unsafe fn init(fb_config: FrameBufferConfig) -> &'static mut Self {
        GRAPHIC = Some(Self::new(fb_config));
        GRAPHIC.as_mut().unwrap()
    }

    pub fn new(fb_config: FrameBufferConfig) -> Self {
        unsafe fn write_pixel_rgb(fb: &mut FrameBuffer, base: usize, rgb: Rgb) {
            fb.write_value(base, [rgb.r, rgb.g, rgb.b]);
        }
        unsafe fn write_pixel_bgr(fb: &mut FrameBuffer, base: usize, rgb: Rgb) {
            fb.write_value(base, [rgb.b, rgb.g, rgb.r]);
        }
        let pixel_writer = match fb_config.pixel_format {
            rusmikan::PixelFormat::RGB => write_pixel_rgb,
            rusmikan::PixelFormat::BGR => write_pixel_bgr,
        };
        Graphic {
            fb_config,
            pixel_writer,
        }
    }

    pub fn write(&mut self, x: usize, y: usize, rgb: Rgb) {
        let pixels_per_scan_line = self.fb_config.pixels_per_scan_line;
        let fb = &mut self.fb_config.frame_buffer;
        unsafe {
            (self.pixel_writer)(fb, (x + pixels_per_scan_line * y) * 4, rgb);
        }
    }

    pub fn write_ascii(&mut self, x: usize, y: usize, c: char, rgb: Rgb) {
        if (c as u32) > 0x7f {
            return;
        }
        let font: [u8; 16] = FONTS[c as usize];
        for dy in 0..16 {
            for dx in 0..8 {
                if (font[dy] << dx & 0x80) != 0 {
                    self.write(x + dx, y + dy, rgb);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        let vert = self.fb_config.vertical_resolution;

        self.clear_line(0, vert);
    }

    pub fn clear_line(&mut self, y: usize, height: usize) {
        let hori = self.fb_config.horizontal_resolution;

        for dy in y..y + height {
            for x in 0..hori {
                self.write(x, dy, BG_COLOR);
            }
        }
    }
}
