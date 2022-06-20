use crate::buffer::CursorPosition;
use crate::editor::Editor;
use crate::modes::Mode;
use crossterm::QueueableCommand;
use crossterm::{
    cursor, queue,
    style::{self, Color, Stylize},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, stdout, Write};

#[derive(Debug, Clone)]
pub struct Cell {
    pub symbol: char,
    pub fg: Color,
    pub bg: Color,
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
            heigth: heigth - 1,
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

impl Editor {
    fn screen_contents(&self) -> ScreenContent {
        let width = self.screen.width;
        let heigth = self.screen.heigth;
        let buffer = self.current_buffer();

        let content = &buffer.content;
        let offset = &buffer.offset;

        let screen_lines = content
            .inner()
            .lines()
            .enumerate()
            .skip(offset.y)
            .take(heigth.into());

        let trimmed_screen_lines = screen_lines.map(|(y, line)| {
            line.chars()
                .enumerate()
                .skip(offset.x)
                .take(width.into())
                .map(move |(x, char)| (x, y, char))
        });

        let screen_content = trimmed_screen_lines
            .map(|lines| {
                lines
                    .map(|(x, y, char)| {
                        let (fg_color, bg_color) = if self.mode == Mode::Visual {
                            let raw_position = buffer.raw_position_coordinates(x, y);
                            if self.last_selection.contains(raw_position) {
                                (Color::White, Color::DarkMagenta)
                            } else {
                                (Color::White, Color::Black)
                            }
                        } else {
                            (Color::White, Color::Black)
                        };
                        Cell {
                            symbol: char,
                            fg: fg_color,
                            bg: bg_color,
                        }
                    })
                    .collect()
            })
            .collect();

        ScreenContent(screen_content)
    }
}

impl Cell {
    fn prepare_display(&self, x: u16, y: u16, screen: &mut Screen) -> Result<()> {
        screen
            .queue(cursor::MoveTo(x, y))?
            .queue(style::PrintStyledContent(
                self.symbol.with(self.fg).on(self.bg),
            ))?;

        if self.symbol == '\t' {
            for _ in 0..3 {
                screen.queue(style::PrintStyledContent(self.symbol.white()))?;
            }
        }

        Ok(())
    }
}

impl Editor {
    pub fn render(&mut self) -> Result<()> {
        let CursorPosition { x, y } = self.current_buffer().screen_cursor_position;
        let screen_contents = self.screen_contents();
        let file_name = self
            .current_buffer()
            .file_name
            .as_ref()
            .and_then(|p| p.to_str().map(ToOwned::to_owned))
            .unwrap_or_default()
            .with(Color::White);

        let screen = &mut self.screen;

        queue!(screen, cursor::Hide)?;

        let mut lines_printed = 0;

        for (y, line) in screen_contents.inner().iter().enumerate() {
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

        // Cleanup of the bottom of the screen
        for y in lines_printed..screen.heigth {
            screen
                .queue(cursor::MoveTo(0, y))?
                .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        }

        screen
            .queue(cursor::MoveTo(0, screen.heigth + 1))?
            .queue(style::PrintStyledContent(file_name))?;

        queue!(screen, cursor::MoveTo(x, y))?;
        queue!(screen, cursor::Show)?;
        screen.flush()?;

        Ok(())
    }
}
