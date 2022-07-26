use crate::actions::Movement;
use crate::buffer::Buffer;
use crate::editor::Editor;
use crate::Direction;
use anyhow::Result;
use itertools::Itertools;

use regex::Regex;

/// This struct behaves like a bidirectional iterator over the completion words
#[derive(Clone, Default, Debug)]
pub struct CompletionWords {
    words: Vec<String>,
    indice: usize,
}

impl CompletionWords {
    fn next(&mut self, direction: Direction) -> &str {
        match direction {
            Direction::Forward => {
                let word = self.words[self.indice].as_str();
                self.indice = (self.indice + 1) % self.words.len();
                word
            }
            Direction::Backward => {
                let word = self.words[self.indice].as_str();

                self.indice = (self.indice as i64 - 1)
                    .rem_euclid(self.words.len() as i64)
                    .try_into()
                    .unwrap();
                word
            }
        }
    }
}

fn get_completion_matches(
    content: &str,
    raw_cursor_position: usize,
    direction: Direction,
) -> CompletionWords {
    let buffer_length = content.len();
    let start_indice = content[..raw_cursor_position]
        .rmatch_indices(|c: char| !c.is_alphanumeric())
        .next()
        .map_or(0, |(i, _)| (i + 1).min(buffer_length));

    let start_pattern = &content[start_indice..raw_cursor_position];

    let re = Regex::new(&format!(r"(\W|^)({start_pattern}\w*)")).unwrap();

    let captures_before = re.captures_iter(&content[..start_indice]);
    let captures_after = re.captures_iter(&content[raw_cursor_position..]);

    let completion_words = captures_after
        .chain(captures_before)
        .map(|capture| {
            let re_match = capture.get(2).unwrap();
            re_match.as_str()
        })
        .filter(|word| !word.is_empty())
        .map(|word| word.to_owned());

    let unique = match direction {
        Direction::Forward => completion_words.unique().collect::<Vec<_>>(),
        Direction::Backward => {
            let mut reversed_words = completion_words.collect::<Vec<_>>();
            reversed_words.reverse();

            let mut unique = reversed_words.into_iter().unique().collect::<Vec<_>>();
            unique.reverse();
            unique
        }
    };

    let len = unique.len();

    CompletionWords {
        words: unique,
        indice: match direction {
            Direction::Backward => len - 1,
            Direction::Forward => 0,
        },
    }
}

impl Buffer {
    fn get_completion_matches(&self, direction: Direction) -> CompletionWords {
        let raw_cursor_position = self.raw_position();
        let content = self.content.inner();
        get_completion_matches(content, raw_cursor_position, direction)
    }
}

impl Editor {
    pub fn insert_completion(&mut self, direction: Direction) -> Result<()> {
        if self.completion_words.is_none() {
            self.completion_words = Some(self.current_buffer().get_completion_matches(direction))
        }

        let completion_words = self.completion_words.as_mut().unwrap();

        let word = completion_words.next(direction).to_owned();
        Movement::Word(-1).delete(self)?;

        self.insert(&word)
    }

    pub fn insert_completion_forward(&mut self) -> Result<()> {
        self.insert_completion(Direction::Forward)
    }

    pub fn insert_completion_backward(&mut self) -> Result<()> {
        self.insert_completion(Direction::Backward)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_test() {
        // Cursor is here            v
        let content = "con,\n\ncont,cconten content; c_onte' ca";

        let completion_matches = get_completion_matches(content, 12, Direction::Forward);

        assert_eq!(
            vec!["conten", "content", "c_onte", "ca", "con", "cont"],
            completion_matches.words
        );
    }

    #[test]
    fn dedup_test() {
        // Cursor is here          v
        let content = r#"con, con con"#;
        let completion_matches_backward = get_completion_matches(content, 10, Direction::Forward);

        assert_eq!(vec!["con"], completion_matches_backward.words);
    }

    #[test]
    fn complete_everything_test() {
        let content = "a, b c d e ";
        let completion_matches_forward = get_completion_matches(content, 11, Direction::Forward);

        assert_eq!(
            vec!["a", "b", "c", "d", "e"],
            completion_matches_forward.words
        );
    }
}
