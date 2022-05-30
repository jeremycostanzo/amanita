use crate::buffer::{Buffer, Cell, Content, Offset};
use crossterm::QueueableCommand;
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{stdout, Stdout, Write};

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

impl Screen {
    pub fn cursor_forward(&mut self) -> Result<()> {
        if self.cursor_position.x + 1 >= self.text_start_x + self.width {
            self.cursor_position.x = self.text_start_x;
            self.cursor_position.y =
                if self.cursor_position.y + 1 >= self.text_start_y + self.heigth {
                    self.text_start_y + self.heigth
                } else {
                    self.cursor_position.y + 1
                }
        } else {
            self.cursor_position.x += 1
        };
        self.terminal
            .queue(cursor::MoveTo(
                self.cursor_position.x,
                self.cursor_position.y,
            ))?
            .flush()?;

        Ok(())
    }

    pub fn cursor_backwards(&mut self) -> Result<()> {
        if self.cursor_position.x < self.text_start_x + 1 {
            self.cursor_position.x = self.text_start_x + self.width;
            self.cursor_position.y = if self.cursor_position.y < self.text_start_y + 1 {
                self.text_start_y
            } else {
                self.cursor_position.y - 1
            }
        } else {
            self.cursor_position.x -= 1
        };
        self.terminal
            .queue(cursor::MoveTo(
                self.cursor_position.x,
                self.cursor_position.y,
            ))?
            .flush()?;

        Ok(())
    }
}

pub struct Screen {
    pub text_start_x: u16,
    pub text_start_y: u16,
    pub width: u16,
    pub heigth: u16,
    pub terminal: Stdout,
    pub cursor_position: ScreenCursorPosition,
}

impl Screen {
    pub fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen)?;
        let (width, heigth) = terminal::size()?;
        let terminal = std::io::stdout();
        let cursor_position = ScreenCursorPosition::default();

        Ok(Screen {
            text_start_x: 0,
            text_start_y: 0,
            width,
            heigth,
            cursor_position,
            terminal,
        })
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        self.terminal
            .execute(terminal::Clear(terminal::ClearType::All))
            .expect("Failed to clear screen");
        terminal::disable_raw_mode().expect("Could not disable the raw mode");
        crossterm::execute!(stdout(), LeaveAlternateScreen)
            .expect("Could not leave alternate screen");
    }
}

fn shrink_buffer_to_screen(
    content: &Content,
    offset: &Offset,
    width: u16,
    heigth: u16,
) -> ScreenContent {
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

impl Cell {
    fn prepare_display(
        &self,
        x: u16,
        y: u16,
        cursor_position: &ScreenCursorPosition,
        output: &mut impl crossterm::QueueableCommand,
    ) -> Result<()> {
        output
            .queue(cursor::MoveTo(x, y))?
            .queue(style::PrintStyledContent(self.symbol.white()))?
            .queue(cursor::MoveTo(cursor_position.x, cursor_position.y))?;
        Ok(())
    }
}

impl ScreenContent {
    fn display(&self, screen: &mut Screen) -> Result<()> {
        screen
            .terminal
            .execute(terminal::Clear(terminal::ClearType::All))?;

        for (y, line) in self.inner().iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                cell.prepare_display(
                    x as u16 + screen.text_start_x,
                    y as u16 + screen.text_start_y,
                    &screen.cursor_position,
                    &mut screen.terminal,
                )?;
            }
        }
        screen.terminal.flush()?;
        Ok(())
    }
}

impl Buffer {
    pub fn render(&self, screen: &mut Screen) -> Result<()> {
        let screen_content = shrink_buffer_to_screen(
            &self.content,
            &self.offset,
            screen.width - screen.text_start_x,
            screen.heigth - screen.text_start_y,
        );

        screen_content.display(screen)?;

        Ok(())
    }
}
