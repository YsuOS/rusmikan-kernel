use crate::graphics::{Graphic,Rgb};
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt::Write;
use crate::graphics::GRAPHIC;

const ROWS: usize = 25;
const COLUMNS: usize = 80;
const WIDTH_PER_WORD: usize = 8;
const HEIGHT_PER_WORD: usize = 16;

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console::new());
}

pub struct Console {
    buffer: [[char; COLUMNS]; ROWS+1],
    column: usize,
    row: usize,
    rgb: Rgb,
}

impl Console {
    pub fn new() -> Self {
        Console {
            buffer: [[0.into(); COLUMNS]; ROWS+1],
            column: 0,
            row: 0,
            rgb: Rgb{r: 0, g: 0, b: 0},
        }
    }

    pub fn put_string(&mut self, graphic: &mut Graphic, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.newline(graphic);
            } else if self.column < COLUMNS {
                if self.row < ROWS {
                    graphic.write_ascii(self.column*WIDTH_PER_WORD, self.row*HEIGHT_PER_WORD, c, self.rgb);
                }
                self.buffer[self.row][self.column] = c;
                self.column += 1; 
            }
        }
    }

    fn newline(&mut self, graphic: &mut Graphic) {
        self.column = 0;
        self.row += 1;
        if self.row > ROWS {
            graphic.clear();

            for row in 0..ROWS {
                for col in 0..COLUMNS {
                   graphic.write_ascii(col*WIDTH_PER_WORD, row*HEIGHT_PER_WORD, self.buffer[row+1][col], self.rgb);
                   self.buffer[row][col] = self.buffer[row+1][col]
                }
            }
            self.row -= 1;
        }
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let graphic = unsafe { GRAPHIC.as_mut().unwrap()};
        self.put_string(graphic, s);
        Ok(())
    }
}