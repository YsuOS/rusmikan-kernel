use rusmikan::FrameBufferConfig;
use uefi::proto::console::gop::FrameBuffer;
use crate::ascii_font::FONTS;

#[derive(Copy,Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Graphic<'gop> {
    fb_config: FrameBufferConfig<'gop>,
    pixel_writer: unsafe fn(&mut FrameBuffer, usize, Rgb),
}

impl<'gop> Graphic<'gop> {
    pub fn new(fb_config: FrameBufferConfig<'gop>) -> Self {
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
            (self.pixel_writer)(fb, (x+pixels_per_scan_line*y)*4, rgb);        
        }
    }

    pub fn write_ascii(&mut self, x: usize, y: usize, c: char, rgb: Rgb) {
        if (c as u32) > 0x7f {
            return;
        }
        let font: [u8;16] = FONTS[c as usize];
        for dy in 0..16 {
            for dx in 0..8 {
                if (font[dy] << dx & 0x80) != 0 {
                    self.write(x+dx, y+dy, rgb);
                }
            }
        }
    }

    pub fn write_string(&mut self, x: usize, y: usize, s: &str, rgb: Rgb) {
        for (i, c) in s.chars().enumerate() {
            self.write_ascii(x+(u8::BITS as usize)*i, y, c, rgb);
        }
    }
}
