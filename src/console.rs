use crate::graphics::{Graphic,Rgb};

const ROWS: usize = 25;
const COLUMNS: usize = 80;
const WIDTH_PER_WORD: usize = 8;
const HEIGHT_PER_WORD: usize = 16;

pub struct Console<'gop> {
    graphic: Graphic<'gop>,
    buffer: [[char; COLUMNS]; ROWS+1],
    column: usize,
    row: usize,
    rgb: Rgb,
}

impl<'gop> Console<'gop> {
    pub fn new(graphic: Graphic<'gop>) -> Self {
        Console {
            graphic: graphic,
            buffer: [[0.into(); COLUMNS]; ROWS+1],
            column: 0,
            row: 0,
            rgb: Rgb{r: 0, g: 0, b: 0},
        }
    }

    pub fn put_string(&mut self, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.newline();
            } else if self.column < COLUMNS {
                if self.row < ROWS {
                    self.graphic.write_ascii(self.column*WIDTH_PER_WORD, self.row*HEIGHT_PER_WORD, c, self.rgb);
                }
                self.buffer[self.row][self.column] = c;
                self.column += 1; 
            }
        }
    }

    fn newline(&mut self) {
        self.column = 0;
        self.row += 1;
        if self.row > ROWS {
            //clear 
            for y in 0..ROWS*HEIGHT_PER_WORD {
                for x in 0..COLUMNS*WIDTH_PER_WORD {
                    self.graphic.write(x, y, Rgb{r: 241, g: 141, b: 0});
                }
            }

            for row in 0..ROWS {
                for col in 0..COLUMNS {
                   self.graphic.write_ascii(col*WIDTH_PER_WORD, row*HEIGHT_PER_WORD, self.buffer[row+1][col], self.rgb);
                   self.buffer[row][col] = self.buffer[row+1][col]
                }
            }
            self.row -= 1;
        }
    }
}
