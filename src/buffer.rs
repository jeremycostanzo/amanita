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

    pub fn inner_mut(&mut self) -> &mut Vec<Vec<Cell>> {
        &mut self.0
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
            .flat_map(|line| {
                line.iter()
                    .map(|cell| cell.symbol)
                    .chain(vec!['\n'].into_iter())
            })
            .collect::<String>()
            .replace("\t\t\t\t", "\t");

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_name)
            .await?;

        file.write_all(buffer_string.as_bytes()).await?;
        Ok(())
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
