use crate::buffer::{Buffer, Cell, Content, Offset};
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand,
};
use std::io::{Stdout, Write};

use anyhow::Result;

pub struct ScreenContent(Vec<Vec<Cell>>);
impl ScreenContent {
    pub fn inner(&self) -> &Vec<Vec<Cell>> {
        &self.0
    }
}

pub struct Screen {
    text_start_x: u16,
    text_start_y: u16,
    width: u16,
    heigth: u16,
    terminal: Stdout,
}

impl Screen {
    pub fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let (width, heigth) = terminal::size()?;
        let terminal = std::io::stdout();

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
        terminal::disable_raw_mode().expect("Could not disable the raw mode");
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
        screen.terminal.flush()?;
        Ok(())
    }
}

impl Buffer {
    pub fn render(&self, mut screen: Screen) -> Result<()> {
        let cursor = &self.cursor;
        let screen_content = shrink_buffer_to_screen(
            &self.content,
            &self.offset,
            screen.width - screen.text_start_x,
            screen.heigth - screen.text_start_y,
        );

        screen_content.display(&mut screen)?;

        Ok(())
    }
}
