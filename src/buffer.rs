use crate::ui::Screen;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;

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

impl Buffer {
    pub fn y(&self) -> usize {
        self.cursor_position.y as usize + self.offset.y
    }
    pub fn x(&self) -> usize {
        self.cursor_position.x as usize + self.offset.x
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

    pub fn move_cursor(&mut self, direction: Direction, screen: &Screen) {
        use Direction::*;
        let inner = self.content.inner();
        match direction {
            Up => {
                if self.cursor_position.y > 0 {
                    self.cursor_position.y -= 1;
                } else {
                    self.offset.y = self.offset.y.saturating_sub(1);
                }

                let new_line_size = inner.get(self.y()).unwrap().len();
                if self.offset.x > new_line_size {
                    self.offset.x = new_line_size.saturating_sub(screen.width as usize);
                }
                if self.x() > new_line_size {
                    self.cursor_position.x =
                        (new_line_size - self.offset.x as usize).try_into().unwrap();
                }
            }
            Left => {
                if self.cursor_position.x > 0 {
                    self.cursor_position.x -= 1;
                } else {
                    self.offset.x.saturating_sub(1);
                }
            }
            Down => {
                if self.y() < (inner.len() - 1) {
                    {
                        if self.cursor_position.y >= (screen.heigth - 1) {
                            self.offset.y += 1;
                        } else {
                            self.cursor_position.y += 1;
                        }
                        let new_line_size = inner.get(self.y()).unwrap().len();
                        if self.offset.x > new_line_size {
                            self.offset.x = new_line_size.saturating_sub(screen.width as usize);
                        }
                        if self.x() > new_line_size {
                            self.cursor_position.x =
                                (new_line_size - self.offset.x as usize).try_into().unwrap();
                        }
                    }
                }
            }
            Right => {
                let current_line_size = inner.get(self.y()).unwrap().len();
                if self.x() < current_line_size {
                    if self.cursor_position.x >= (screen.width - 1) {
                        self.offset.x += 1
                    } else {
                        self.cursor_position.x += 1
                    }
                }
            }
        };
    }

    pub fn delete_char(&mut self) {
        let x = &mut self.cursor_position.x;
        let y = &mut self.cursor_position.y;
        let inner = &mut self.content.0;
        if *x == 0 {
            if *y != 0 {
                let current_line = inner.get(*y).unwrap().clone();
                let previous_line = inner.get_mut(*y - 1).unwrap();
                previous_line.extend(current_line);
                inner.remove(*y);
                *y -= 1;
                *x = inner.get(*y).unwrap().len();
            }
        } else {
            inner.get_mut(*y).unwrap().remove((*x) - 1);
            *x -= 1;
        }
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
