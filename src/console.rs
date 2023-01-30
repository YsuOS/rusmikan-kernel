use crate::graphics::{Graphic, Rgb, GRAPHIC};
use core::fmt::Write;
use spin::Mutex;

const ROWS: usize = 25;
const COLUMNS: usize = 80;
const WIDTH_PER_WORD: usize = 8;
const HEIGHT_PER_WORD: usize = 16;

pub static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

pub struct Console {
    buffer: [[char; COLUMNS]; ROWS],
    column: usize,
    row: usize,
    rgb: Rgb,
}

impl Console {
    const fn new() -> Self {
        Console {
            buffer: [[0 as char; COLUMNS]; ROWS],
            column: 0,
            row: 0,
            rgb: Rgb { r: 0, g: 0, b: 0 },
        }
    }

    fn put_string(&mut self, graphic: &mut Graphic, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.newline(graphic);
            } else if self.column < COLUMNS {
                if self.row < ROWS {
                    graphic.write_ascii(
                        self.column * WIDTH_PER_WORD,
                        self.row * HEIGHT_PER_WORD,
                        c,
                        self.rgb,
                    );
                }
                self.buffer[self.row][self.column] = c;
                self.column += 1;
            }
        }
    }

    fn newline(&mut self, graphic: &mut Graphic) {
        self.column = 0;
        self.row += 1;
        if self.row >= ROWS {
            for row in 0..ROWS {
                graphic.clear_line(row * HEIGHT_PER_WORD, HEIGHT_PER_WORD);
                for col in 0..COLUMNS {
                    if row + 1 == ROWS {
                        self.buffer[row][col] = ' ';
                    } else {
                        self.buffer[row][col] = self.buffer[row + 1][col];
                        graphic.write_ascii(
                            col * WIDTH_PER_WORD,
                            row * HEIGHT_PER_WORD,
                            self.buffer[row + 1][col],
                            self.rgb,
                        );
                    }
                }
            }
            self.row -= 1;
        }
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let graphic = unsafe { GRAPHIC.as_mut().unwrap() };
        self.put_string(graphic, s);
        Ok(())
    }
}
