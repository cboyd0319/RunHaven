//! VT100 snapshot backend adapted from `openai/codex`
//! `codex-rs/tui/src/test_backend.rs`.
//!
//! Licensed under Apache-2.0; see `THIRD_PARTY_NOTICES.md`.

use std::fmt;
use std::io::{self, Write};

use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};
use ratatui::prelude::CrosstermBackend;

pub(crate) struct Vt100Backend {
    crossterm_backend: CrosstermBackend<vt100::Parser>,
}

impl Vt100Backend {
    pub(crate) fn new(width: u16, height: u16) -> Self {
        ratatui::crossterm::style::force_color_output(true);
        Self {
            crossterm_backend: CrosstermBackend::new(vt100::Parser::new(height, width, 0)),
        }
    }

    pub(crate) fn vt100(&self) -> &vt100::Parser {
        self.crossterm_backend.writer()
    }
}

impl Write for Vt100Backend {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.crossterm_backend.writer_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.crossterm_backend.writer_mut().flush()
    }
}

impl fmt::Display for Vt100Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.vt100().screen().contents())
    }
}

impl Backend for Vt100Backend {
    type Error = io::Error;

    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        self.crossterm_backend.draw(content)
    }

    fn append_lines(&mut self, n: u16) -> io::Result<()> {
        self.crossterm_backend.append_lines(n)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.crossterm_backend.hide_cursor()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.crossterm_backend.show_cursor()
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(self.vt100().screen().cursor_position().into())
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        self.crossterm_backend.set_cursor_position(position)
    }

    fn clear(&mut self) -> io::Result<()> {
        self.crossterm_backend.clear()
    }

    fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
        self.crossterm_backend.clear_region(clear_type)
    }

    fn size(&self) -> io::Result<Size> {
        let (rows, cols) = self.vt100().screen().size();
        Ok(Size::new(cols, rows))
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: self.size()?,
            pixels: Size {
                width: 640,
                height: 480,
            },
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.crossterm_backend.writer_mut().flush()
    }
}
