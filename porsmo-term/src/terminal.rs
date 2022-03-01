use std::{
    fmt::Display,
    io::{stdout, Stdout, Write},
};
use termion::{
    clear, color, cursor,
    raw::{IntoRawMode, RawTerminal},
};

pub struct RawTerm<T>
where
    T: Write,
{
    out: RawTerminal<T>,
    line_no: u16,
}

#[allow(dead_code)]
impl<T: Write> RawTerm<T> {
    pub fn new(out: T) -> Result<Self, std::io::Error> {
        Ok(Self {
            out: out.into_raw_mode()?,
            line_no: 0,
        })
    }

    pub fn get_out(&mut self) -> &mut RawTerminal<T> {
        &mut self.out
    }

    pub fn set_color(&mut self, color: impl color::Color) -> Result<(), std::io::Error> {
        write!(self.out, "{}", color::Fg(color))
    }

    pub fn write_raw_line(&mut self, show: impl Display, line: u16) -> Result<(), std::io::Error> {
        write!(
            self.out,
            "{goto}{show}",
            goto = cursor::Goto(1, line),
            show = show
        )
    }

    pub fn write_line(&mut self, show: impl Display) -> Result<(), std::io::Error> {
        self.line_no += 1;
        self.write_raw_line(show, self.line_no)
    }

    pub fn reset_line_no(&mut self) {
        self.line_no = 0;
    }

    pub fn clear(&mut self) -> Result<(), std::io::Error> {
        self.reset_line_no();
        write!(self.out, "{}", clear::All)
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.out.flush()
    }

    pub fn reset(&mut self) -> Result<(), std::io::Error> {
        self.reset_line_no();
        write!(
            self.out,
            "{top}{clear}{show}{color}",
            top = cursor::Goto(1, 1),
            clear = clear::All,
            color = color::Fg(color::Reset),
            show = termion::cursor::Show
        )?;

        self.out.flush()
    }

    pub fn destroy(self) {}
}

impl Default for RawTerm<Stdout> {
    fn default() -> Self {
        Self {
            out: stdout().into_raw_mode().unwrap(),
            line_no: 0,
        }
    }
}

impl<T: Write> Drop for RawTerm<T> {
    fn drop(&mut self) {
        self.reset().ok();
    }
}
