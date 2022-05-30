use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;

use anyhow::Result;

#[derive(Debug, Default, Clone)]
pub struct CursorPosition {
    pub x: usize,
    pub y: usize,
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

    pub fn insert(&mut self, c: char) -> anyhow::Result<()> {
        let CursorPosition { x, y } = self.cursor_position;
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

        self.cursor_position.x += 1;

        Ok(())
    }
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
