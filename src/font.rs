use rusmikan::FrameBufferConfig;
use crate::graphics::{PixelWriter,Rgb};

const FONT_A: [u8; 16] = [
    0b00000000,
    0b00011000,
    0b00011000,
    0b00011000,
    0b00011000,
    0b00100100,
    0b00100100,
    0b00100100,
    0b00100100,
    0b01111110,
    0b01000010,
    0b01000010,
    0b01000010,
    0b11100111,
    0b00000000,
    0b00000000,
];

pub fn write_ascii(pixel_writer: &dyn PixelWriter, fb_config: &mut FrameBufferConfig, x: usize, y: usize, c: char, rgb: Rgb) {
    if c != 'A' {
        return;
    }
    for dy in 0..16 {
        for dx in 0..8 {
            if (FONT_A[dy] << dx & 0x80) != 0 {
                unsafe {
                    pixel_writer.write(fb_config, x+dx, y+dy, rgb);
                }
            }
        }
    }
}
