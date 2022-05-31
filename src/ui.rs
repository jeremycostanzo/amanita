use crate::buffer::{Buffer, Cell, Content, Offset};
use crossterm::cursor::RestorePosition;
use crossterm::cursor::SavePosition;
use crossterm::QueueableCommand;
use crossterm::{
    cursor::{self},
    queue,
    style::{self, Stylize},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, stdout, Write};

use anyhow::Result;

pub struct ScreenContent(Vec<Vec<Cell>>);
impl ScreenContent {
    pub fn inner(&self) -> &Vec<Vec<Cell>> {
        &self.0
    }
}
#[derive(Debug, Default)]
pub struct ScreenCursorPosition {
    pub x: u16,
    pub y: u16,
}

pub struct Screen {
    pub text_start_x: u16,
    pub text_start_y: u16,
    pub width: u16,
    pub heigth: u16,
    pub terminal: String,
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
        let mut screen_content = Vec::new();
        for y in offset.y..(offset.y + heigth as usize) {
            let mut line = Vec::new();
            for x in offset.x..(offset.x + width as usize) {
                line.push(
                    content
                        .inner()
                        .get(y)
                        .and_then(|content_line| content_line.get(x))
                        .cloned()
                        .unwrap_or_default(),
                );
            }
            screen_content.push(line);
        }
        ScreenContent(screen_content)
    }
}

impl Cell {
    fn prepare_display(&self, x: u16, y: u16, screen: &mut Screen) -> Result<()> {
        screen
            .queue(cursor::MoveTo(x, y))?
            .queue(style::PrintStyledContent(self.symbol.white()))?;
        Ok(())
    }
}

impl ScreenContent {
    fn display(&self, screen: &mut Screen) -> Result<()> {
        for (y, line) in self.inner().iter().enumerate() {
            screen.queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
            for (x, cell) in line.iter().enumerate() {
                cell.prepare_display(
                    x as u16 + screen.text_start_x,
                    y as u16 + screen.text_start_y,
                    screen,
                )?;
            }
        }
        Ok(())
    }
}

impl Buffer {
    pub fn screen_cursor_position(&self) -> ScreenCursorPosition {
        let x = (self.cursor_position.x - self.offset.x).try_into().unwrap();
        let y = (self.cursor_position.y - self.offset.y).try_into().unwrap();
        ScreenCursorPosition { x, y }
    }
    pub fn render(&self, screen: &mut Screen) -> Result<()> {
        let screen_content = self.display_on_screen(
            screen.width - screen.text_start_x,
            screen.heigth - screen.text_start_y,
        );

        queue!(screen, cursor::Hide)?;
        screen_content.display(screen)?;
        let ScreenCursorPosition { x, y } = self.screen_cursor_position();
        queue!(screen, cursor::MoveTo(x, y))?;
        queue!(screen, cursor::Show)?;
        screen.flush()?;

        Ok(())
    }
}
