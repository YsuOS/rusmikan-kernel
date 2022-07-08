use rusmikan::FrameBufferConfig;
use crate::graphics::{PixelWriter,Rgb};
use crate::ascii_font::FONTS;

// x and y is a point in pixel, not column and row in string
pub fn write_string(pixel_writer: &dyn PixelWriter, fb_config: &mut FrameBufferConfig, x: usize, y: usize, s: &str, rgb: Rgb) {
    for (i, c) in s.chars().enumerate() {
        write_ascii(pixel_writer, fb_config, x+8*i, y, c, rgb);
    }
}

pub fn write_ascii(pixel_writer: &dyn PixelWriter, fb_config: &mut FrameBufferConfig, x: usize, y: usize, c: char, rgb: Rgb) {
    if (c as u32) > 0x7f {
        return;
    }
    let font: [u8;16] = FONTS[c as usize];
    for dy in 0..16 {
        for dx in 0..8 {
            if (font[dy] << dx & 0x80) != 0 {
                unsafe {
                    pixel_writer.write(fb_config, x+dx, y+dy, rgb);
                }
            }
        }
    }
}
