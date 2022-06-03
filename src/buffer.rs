use crate::ui::Screen;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use anyhow::Result;

#[derive(Debug, Default, Clone)]
pub struct CursorPosition {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Default, Clone)]
pub struct Cell {
    pub symbol: char,
}

#[derive(Debug, Default)]
pub struct Offset {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Default)]
pub struct Content(Vec<Vec<Cell>>);
impl Content {
    pub fn inner(&self) -> &Vec<Vec<Cell>> {
        &self.0
    }
}

impl FromStr for Content {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Content(
            s.lines()
                .into_iter()
                .map(|line| {
                    line.chars()
                        .into_iter()
                        .map(|char| Cell { symbol: char })
                        .collect()
                })
                .collect(),
        ))
    }
}

#[derive(Debug, Default)]
pub struct Buffer {
    pub content: Content,
    pub cursor_position: CursorPosition,
    pub offset: Offset,
    pub file_name: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct NoFileName;

impl std::fmt::Display for NoFileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No file name provided")
    }
}
impl std::error::Error for NoFileName {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl Buffer {
    pub fn y(&self) -> usize {
        self.cursor_position.y as usize + self.offset.y
    }
    pub fn x(&self) -> usize {
        self.cursor_position.x as usize + self.offset.x
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let file_name = self.file_name.as_ref().ok_or(NoFileName)?;
        let buffer_string: String = self
            .content
            .inner()
            .iter()
            .flat_map(|line| line.iter().map(|cell| cell.symbol))
            .collect();

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_name)
            .await?;

        file.write_all(buffer_string.as_bytes()).await?;
        Ok(())
    }

    // Used after a move of cursor, to ensure that the cursor never goes out of a line
    fn adjust_x(&mut self, screen: &Screen) {
        let new_line_size = self.content.inner().get(self.y()).unwrap().len();
        if self.offset.x > new_line_size {
            self.offset.x = new_line_size.saturating_sub(screen.width as usize);
        }
        if self.x() > new_line_size {
            self.cursor_position.x = (new_line_size - self.offset.x as usize).try_into().unwrap();
        }
    }

    fn sub_any(cursor_position: &mut u16, offset: &mut usize, value: usize) {
        let to_remove_to_cursor_position: u16 =
            value.min(*cursor_position as usize).try_into().unwrap();
        let to_remove_to_offset = value - to_remove_to_cursor_position as usize;

        *cursor_position -= to_remove_to_cursor_position;
        *offset = (*offset).saturating_sub(to_remove_to_offset);
    }

    fn sub_x(&mut self, x: usize) {
        Buffer::sub_any(&mut self.cursor_position.x, &mut self.offset.x, x);
    }

    fn sub_y(&mut self, y: usize, screen: &Screen) {
        Buffer::sub_any(&mut self.cursor_position.y, &mut self.offset.y, y);
        self.adjust_x(screen);
    }

    fn add_any(cursor_position: &mut u16, offset: &mut usize, box_size: u16, value: usize) {
        let to_add_to_cursor_position: u16 = (value
            .min((box_size - *cursor_position - 1) as usize))
        .try_into()
        .unwrap();
        let to_add_to_offset = value - to_add_to_cursor_position as usize;

        *cursor_position += to_add_to_cursor_position;
        *offset += to_add_to_offset;
    }

    fn add_x(&mut self, x: usize, screen: &Screen) {
        Buffer::add_any(
            &mut self.cursor_position.x,
            &mut self.offset.x,
            screen.width,
            x,
        );
        self.adjust_x(screen);
    }

    fn add_y(&mut self, y: usize, screen: &Screen) {
        let to_add = y.min(self.content.inner().len() - self.y() - 1);
        Buffer::add_any(
            &mut self.cursor_position.y,
            &mut self.offset.y,
            screen.heigth,
            to_add,
        );
        self.adjust_x(screen);
    }

    pub async fn from_file(path: &Path) -> Self {
        let content = fs::read_to_string(path)
            .await
            .unwrap_or_else(|_| Default::default())
            .parse()
            .unwrap();

        Buffer {
            content,
            cursor_position: Default::default(),
            offset: Default::default(),
            file_name: Some(path.to_owned()),
        }
    }

    pub fn insert(&mut self, c: char, screen: &Screen) {
        let (x, y) = (self.x(), self.y());
        let content = &mut self.content.0;
        let line = content.get_mut(y);
        let line = match line {
            Some(line) => line,
            None => match content.last_mut() {
                Some(line) => line,
                None => {
                    content.push(Vec::new());
                    content.get_mut(0).unwrap()
                }
            },
        };

        line.insert(x, Cell { symbol: c });

        if self.cursor_position.x == screen.width {
            self.offset.x += 1;
        } else {
            self.cursor_position.x += 1;
        }
    }

    pub fn move_cursor(&mut self, direction: Direction, value: usize, screen: &Screen) {
        use Direction::*;
        match direction {
            Up => self.sub_y(value, screen),
            Left => self.sub_x(value),
            Down => self.add_y(value, screen),
            Right => self.add_x(value, screen),
        };
    }

    pub fn delete_char(&mut self, screen: &Screen) {
        let x = self.x();
        let y = self.y();
        let inner = &mut self.content.0;
        if x == 0 {
            if y != 0 {
                let current_line = inner.get(y).unwrap().clone();
                let previous_line = inner.get_mut(y - 1).unwrap();
                let previous_line_length = previous_line.len();
                previous_line.extend(current_line);
                inner.remove(y);
                self.move_cursor(Direction::Up, 1, screen);
                self.move_cursor(Direction::Right, previous_line_length, screen);
            }
        } else {
            inner.get_mut(y).unwrap().remove((x) - 1);
            self.move_cursor(Direction::Left, 1, screen);
        }
    }

    pub fn add_new_line(&mut self, screen: &Screen) {
        let y = self.y();
        let x = self.x();
        let inner = &mut self.content.0;
        let current_line = inner.get_mut(y).unwrap();
        let to_add = current_line.drain(x..).collect();
        inner.insert(y + 1, to_add);
        self.move_cursor(Direction::Down, 1, screen);
        self.offset.x = 0;
        self.cursor_position.x = 0;
    }
}

pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_file() {
        let buffer = Buffer::from_file(Path::new("src/buffer.rs")).await;
        dbg!(buffer);
    }
}
