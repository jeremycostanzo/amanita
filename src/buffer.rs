use crate::OutOfBounds;
use anyhow::Context;
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
pub struct Offset {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Content(String);
impl Content {
    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl FromStr for Content {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Content(s.to_owned()))
    }
}

#[derive(Debug, Default, Clone)]
pub struct Buffer {
    pub content: Content,
    pub screen_cursor_position: CursorPosition,
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
        self.screen_cursor_position.y as usize + self.offset.y
    }
    pub fn x(&self) -> usize {
        self.screen_cursor_position.x as usize + self.offset.x
    }

    pub fn raw_position(&self) -> usize {
        self.raw_position_coordinates(self.x(), self.y())
    }

    pub fn raw_position_coordinates(&self, x: usize, y: usize) -> usize {
        let lines = &mut self.content.inner().lines();

        let beginning_lines = lines.take(y);

        let beginning_count =
            beginning_lines.fold(0, |character_count, line| character_count + line.len() + 1);

        beginning_count + x
    }

    pub fn current_line(&self) -> Result<&str> {
        self.content
            .inner()
            .lines()
            .nth(self.y())
            .ok_or_else(|| OutOfBounds(self.y()))
            .context("Current line")
    }

    pub fn current_line_length(&self) -> Result<usize> {
        Ok(self.current_line().context("Current line length")?.len())
    }

    pub fn lines_count(&self) -> Result<usize> {
        Ok(self.content.inner().lines().count())
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let file_name = self
            .file_name
            .as_ref()
            .ok_or(NoFileName)
            .context("Tried to save with no file name")?;
        let buffer_string: String = self.content.inner().replace("\t\t\t\t", "\t");

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_name)
            .await?;

        file.write_all(buffer_string.as_bytes()).await?;
        Ok(())
    }

    pub async fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .await
            .unwrap_or_else(|_| Default::default())
            .parse()?;

        Ok(Buffer {
            content,
            screen_cursor_position: Default::default(),
            offset: Default::default(),
            file_name: Some(path.to_owned()),
        })
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum CharacterType {
    Word,
    Punctuation,
    Other,
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl From<char> for CharacterType {
    fn from(c: char) -> Self {
        if is_word_char(c) {
            CharacterType::Word
        } else if c.is_ascii_punctuation() {
            CharacterType::Punctuation
        } else {
            CharacterType::Other
        }
    }
}

impl Buffer {
    fn next_word_index(&self, position: usize) -> usize {
        let inner = self.content.inner();
        let mut chars = inner.chars().skip(position);
        let char_type_on_cursor: CharacterType = chars.next().unwrap().into();

        let mut went_through_other = false;

        for index in (position + 1)..inner.len() {
            let char_type_on_index: CharacterType = chars.next().unwrap().into();
            match (char_type_on_cursor, char_type_on_index) {
                (CharacterType::Word, CharacterType::Punctuation)
                | (CharacterType::Punctuation, CharacterType::Word)
                | (CharacterType::Other, CharacterType::Word)
                | (CharacterType::Other, CharacterType::Punctuation) => return index,
                (_, CharacterType::Other) => went_through_other = true,
                (CharacterType::Word, CharacterType::Word)
                | (CharacterType::Punctuation, CharacterType::Punctuation) => {
                    if went_through_other {
                        return index;
                    }
                }
            }
        }

        inner.len() - 1
    }

    fn previous_word_index(&self, position: usize) -> usize {
        if position < 2 {
            return 0;
        }

        let inner = self.content.inner();
        let mut chars = inner[..position].chars().rev();
        let char_type_before_cursor: CharacterType = chars.next().unwrap().into();

        let mut locked_character_type = char_type_before_cursor;

        for index in (0..(position - 2)).rev() {
            let char_type_on_index: CharacterType = chars.next().unwrap().into();
            match (locked_character_type, char_type_on_index) {
                (CharacterType::Word, CharacterType::Punctuation)
                | (CharacterType::Punctuation, CharacterType::Word)
                | (CharacterType::Word, CharacterType::Other)
                | (CharacterType::Punctuation, CharacterType::Other) => return index + 2,

                (CharacterType::Other, CharacterType::Other)
                | (CharacterType::Word, CharacterType::Word)
                | (CharacterType::Punctuation, CharacterType::Punctuation) => {}

                (CharacterType::Other, alpha_or_punctuation) => {
                    locked_character_type = alpha_or_punctuation
                }
            }
        }
        0
    }

    pub fn nth_word_index(&self, delta: i64) -> usize {
        let mut position = self.raw_position();
        match delta.cmp(&0) {
            std::cmp::Ordering::Less => {
                for _ in 0..(-delta) {
                    position = self.previous_word_index(position);
                }
                position
            }
            std::cmp::Ordering::Equal => position,
            std::cmp::Ordering::Greater => {
                for _ in 0..delta {
                    position = self.next_word_index(position);
                }
                position
            }
        }
    }

    fn previous_word_end_index(&self, position: usize) -> usize {
        let inner = self.content.inner();
        let mut chars = inner[0..=position].chars().rev().enumerate();
        let initial_char_type: Option<CharacterType> = chars.next().map(|(_, c)| c.into());
        if initial_char_type.is_none() {
            return 0;
        }
        let mut char_index = 0;
        let mut char_type_on_cursor: Option<CharacterType> = chars.next().map(|(i, c)| {
            char_index = i;
            c.into()
        });
        while char_type_on_cursor == initial_char_type {
            char_type_on_cursor = chars.next().map(|(i, c)| {
                char_index = i;
                c.into()
            });
        }

        match char_type_on_cursor {
            None => 0,
            Some(char_type) => {
                if char_type != CharacterType::Other {
                    position.saturating_sub(char_index)
                } else {
                    chars
                        .find(|(_, c)| {
                            let current_char_type: CharacterType = (*c).into();
                            current_char_type != CharacterType::Other
                        })
                        .map(|(i, _)| position.saturating_sub(i))
                        .unwrap_or(0)
                }
            }
        }
    }

    fn next_word_end_index(&self, position: usize) -> usize {
        let inner = self.content.inner();
        let mut chars = inner.chars().enumerate().skip(position + 1);
        let mut char_type_on_cursor: Option<CharacterType> = chars.next().map(|(_, c)| c.into());
        while char_type_on_cursor == Some(CharacterType::Other) {
            char_type_on_cursor = chars.next().map(|(_, c)| c.into());
        }

        match char_type_on_cursor {
            None => inner.len() - 1,
            Some(char_type) => chars
                .find(|(_, c)| {
                    let current_char_type: CharacterType = (*c).into();
                    current_char_type != char_type
                })
                .map(|(i, _)| i.saturating_sub(1))
                .unwrap_or(inner.len() - 1),
        }
    }

    pub fn nth_word_end_index(&self, delta: i64) -> usize {
        let mut position = self.raw_position();
        match delta.cmp(&0) {
            std::cmp::Ordering::Less => {
                for _ in 0..(-delta) {
                    position = self.previous_word_end_index(position);
                }
                position
            }
            std::cmp::Ordering::Equal => position,
            std::cmp::Ordering::Greater => {
                for _ in 0..delta {
                    position = self.next_word_end_index(position);
                }
                position
            }
        }
    }

    pub fn next_char_index(&self, char: char, delta: i64) -> Option<usize> {
        let content = self.content.inner();
        let current_position = self.raw_position();
        let target = if delta >= 0 {
            let slice_start = (current_position + 1).min(content.len());
            let matches = &mut content[slice_start..content.len()].match_indices(char);
            matches
                .nth(delta as usize)
                .map(|(indice, _)| indice + slice_start)
        } else {
            let matches = &mut content[0..current_position].match_indices(char).rev();
            matches
                .nth((-(delta as i64) - 1) as usize)
                .map(|(indice, _)| indice)
        };

        target
    }

    pub fn search(&self, substring: &str, direction: crate::Direction) -> Option<usize> {
        let content = self.content.inner();
        let position = self.raw_position();

        use crate::Direction;
        match direction {
            Direction::Forward => content[position..].find(substring),
            Direction::Backward => content[position..].rfind(substring),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_file() {
        let buffer = Buffer::from_file(Path::new("src/buffer.rs")).await;
        dbg!(buffer.unwrap());
    }
}
