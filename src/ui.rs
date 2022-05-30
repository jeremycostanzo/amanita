use crate::buffer::{Buffer, Cell, Content, Offset};
use crossterm::QueueableCommand;
use crossterm::{
    cursor::{self, position},
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

pub struct Screen {
    pub text_start_x: u16,
    pub text_start_y: u16,
    pub width: u16,
    pub heigth: u16,
    pub terminal: Stdout,
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
        output: &mut impl crossterm::QueueableCommand,
    ) -> Result<()> {
        output
            .queue(cursor::MoveTo(x, y))?
            .queue(style::PrintStyledContent(self.symbol.white()))?;
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
                    &mut screen.terminal,
                )?;
            }
        }
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

        let screen_cursor_position = position()?;
        screen_content.display(screen)?;
        screen
            .terminal
            .queue(cursor::MoveTo(
                screen_cursor_position.0,
                screen_cursor_position.1,
            ))?
            .flush()?;

        Ok(())
    }
}
