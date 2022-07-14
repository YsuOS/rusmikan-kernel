use rusmikan::FrameBufferConfig;
use crate::graphics::{PixelWriter,Rgb};
use crate::font::{write_ascii,write_string};

const ROWS: usize = 25;
const COLUMNS: usize = 80;

static mut CURSOR_COLUMN: usize = 0;
static mut CURSOR_ROW: usize = 0;
static mut BUF:[[char; COLUMNS]; ROWS+10] = [[' '; COLUMNS]; ROWS+10];

pub fn put_string(pixel_writer: &dyn PixelWriter, fb_config: &mut FrameBufferConfig, s: &str) {
    unsafe {
        for c in s.chars() {
            if c == '\n' {
                CURSOR_COLUMN = 0;
                CURSOR_ROW += 1;
                if CURSOR_ROW > ROWS {
                    for y in 0..16*ROWS {
                        for x in 0..8*COLUMNS {
                            pixel_writer.write(fb_config, x, y, Rgb{r: 241, g: 141, b: 0});
                        }
                    }
                    write_string(pixel_writer, fb_config, 0, 24*16, "__________", Rgb {r: 0, g: 0, b: 255});
                    for row in 0..ROWS {
                        for col in 0..COLUMNS {
                            write_ascii(pixel_writer, fb_config, 8*col, 16*row, BUF[row+(CURSOR_ROW-ROWS)][col], Rgb{r: 0, g: 255, b: 0});
                        }
                    }
                } 
            } else {
                if CURSOR_COLUMN < COLUMNS {
                    write_ascii(pixel_writer, fb_config, 8*CURSOR_COLUMN, 16*CURSOR_ROW, c, Rgb{r: 0, g: 0, b: 0});
                    BUF[CURSOR_ROW][CURSOR_COLUMN] = c;
                }
                CURSOR_COLUMN += 1;
            }
        }
    }
}
