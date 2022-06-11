use crate::buffer::Buffer;
use crate::buffer::CursorPosition;
use crate::Editor;
use crossterm::QueueableCommand;
use crossterm::{
    cursor, queue,
    style::{self, Stylize},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, stdout, Write};

#[derive(Debug, Default, Clone)]
pub struct Cell {
    pub symbol: char,
}

impl From<char> for Cell {
    fn from(symbol: char) -> Self {
        Self { symbol }
    }
}

use anyhow::Result;

pub struct ScreenContent(Vec<Vec<Cell>>);
impl ScreenContent {
    pub fn inner(&self) -> &Vec<Vec<Cell>> {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Screen {
    pub text_start_x: u16,
    pub text_start_y: u16,
    pub width: u16,
    pub heigth: u16,
    terminal: String,
}

impl Screen {
    pub fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen)?;
        let (width, heigth) = terminal::size()?;
        let terminal = String::new();

        Ok(Screen {
            text_start_x: 0,
            text_start_y: 0,
            width,
            heigth,
            terminal,
        })
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl io::Write for Screen {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.terminal.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(io::ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out = write!(stdout(), "{}", self.terminal);
        stdout().flush()?;
        self.terminal.clear();
        out
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.execute(terminal::Clear(terminal::ClearType::All))
            .expect("Failed to clear screen");
        terminal::disable_raw_mode().expect("Could not disable the raw mode");
        crossterm::execute!(stdout(), LeaveAlternateScreen)
            .expect("Could not leave alternate screen");
    }
}

impl Buffer {
    fn display_on_screen(&self, width: u16, heigth: u16) -> ScreenContent {
        let content = &self.content;
        let offset = &self.offset;

        let screen_lines = content.inner().lines().skip(offset.y).take(heigth.into());

        let trimmed_screen_lines =
            screen_lines.map(|line| line.chars().skip(offset.x).take(width.into()));

        let screen_content = trimmed_screen_lines
            .map(|chars| chars.map(Into::into).collect())
            .collect();

        ScreenContent(screen_content)
    }
}

impl Cell {
    fn prepare_display(&self, x: u16, y: u16, screen: &mut Screen) -> Result<()> {
        screen
            .queue(cursor::MoveTo(x, y))?
            .queue(style::PrintStyledContent(self.symbol.white()))?;

        if self.symbol == '\t' {
            for _ in 0..3 {
                screen.queue(style::PrintStyledContent(self.symbol.white()))?;
            }
        }

        Ok(())
    }
}

impl ScreenContent {
    fn display(&self, screen: &mut Screen) -> Result<()> {
        let mut lines_printed = 0;
        for (y, line) in self.inner().iter().enumerate() {
            let y = y as u16 + screen.text_start_y;
            screen
                .queue(cursor::MoveTo(0, y))?
                .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;

            for (x, cell) in line.iter().enumerate() {
                let x = x as u16 + screen.text_start_x;
                screen.queue(cursor::MoveTo(x, y))?;
                cell.prepare_display(x, y, screen)?;
            }
            lines_printed += 1;
        }
        for y in lines_printed..screen.heigth {
            screen
                .queue(cursor::MoveTo(0, y))?
                .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        }
        Ok(())
    }
}

impl Editor {
    pub fn render(&mut self) -> Result<()> {
        let buffer = self.current_buffer();
        let screen = self.screen();
        let screen_content = buffer.display_on_screen(
            screen.width - screen.text_start_x,
            screen.heigth - screen.text_start_y,
        );

        let CursorPosition { x, y } = buffer.screen_cursor_position;

        let screen_mut = self.screen_mut();

        queue!(screen_mut, cursor::Hide)?;
        screen_content.display(screen_mut)?;
        queue!(screen_mut, cursor::MoveTo(x, y))?;
        queue!(screen_mut, cursor::Show)?;
        screen_mut.flush()?;

        Ok(())
    }
}
