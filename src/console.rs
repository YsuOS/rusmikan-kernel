use crate::graphics::{Graphic, Rgb, GRAPHIC};
use core::fmt::Write;
use spin::{Lazy, Mutex};

const ROWS: usize = 25;
const COLUMNS: usize = 80;
const WIDTH_PER_WORD: usize = 8;
const HEIGHT_PER_WORD: usize = 16;

pub static CONSOLE: Lazy<Mutex<Console>> = Lazy::new(|| Mutex::new(Console::new()));

pub struct Console {
    buffer: [[char; COLUMNS]; ROWS],
    column: usize,
    row: usize,
    rgb: Rgb,
    graphic: &'static Mutex<Graphic>,
}

unsafe impl Sync for Console {}
unsafe impl Send for Console {}

impl Console {
    fn new() -> Self {
        Console {
            buffer: [[0 as char; COLUMNS]; ROWS],
            column: 0,
            row: 0,
            rgb: Rgb { r: 0, g: 0, b: 0 },
            graphic: GRAPHIC.get().unwrap(),
        }
    }

    fn put_string(&mut self, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.newline();
            } else if self.column < COLUMNS {
                if self.row < ROWS {
                    self.graphic.lock().write_ascii(
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

    fn newline(&mut self) {
        self.column = 0;
        self.row += 1;
        if self.row >= ROWS {
            for row in 0..ROWS {
                self.graphic
                    .lock()
                    .clear_line(row * HEIGHT_PER_WORD, HEIGHT_PER_WORD);
                for col in 0..COLUMNS {
                    if row + 1 == ROWS {
                        self.buffer[row][col] = ' ';
                    } else {
                        self.buffer[row][col] = self.buffer[row + 1][col];
                        self.graphic.lock().write_ascii(
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
        self.put_string(s);
        Ok(())
    }
}
