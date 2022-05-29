use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs::File;

#[derive(Debug, Default, Clone)]
pub struct Cell {
    pub symbol: char,
}

#[derive(Debug, Default)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
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
    pub cursor: Cursor,
    pub offset: Offset,
    pub file_name: Option<PathBuf>,
}

impl Buffer {
    pub async fn from_file(path: &Path) -> Self {
        let content = tokio::fs::read_to_string(path)
            .await
            .unwrap_or_else(|_| Default::default())
            .parse()
            .unwrap();

        Buffer {
            content,
            cursor: Default::default(),
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
