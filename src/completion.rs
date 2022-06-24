use crate::editor::Editor;
use itertools::Itertools;
use std::collections::BTreeSet;

use regex::Regex;

enum CompletionDirection {
    Backward,
    Forward,
}

fn get_completion_matches(
    content: &str,
    raw_cursor_position: usize,
    direction: CompletionDirection,
) -> Vec<&str> {
    let buffer_length = content.len();
    let start_indice = content[..raw_cursor_position]
        .rmatch_indices(|c: char| !c.is_alphanumeric())
        .next()
        .map_or(0, |(i, _)| (i + 1).min(buffer_length));

    let start_pattern = &content[start_indice..raw_cursor_position];

    let re = Regex::new(&format!(r"\W?({start_pattern}\w*)")).unwrap();

    let captures_before = re.captures_iter(&content[..start_indice]).map(|capture| {
        let re_match = capture.get(1).unwrap();
        re_match.as_str()
    });

    let captures_after = re
        .captures_iter(&content[raw_cursor_position..])
        .map(|capture| {
            let re_match = capture.get(1).unwrap();
            re_match.as_str()
        });

    let mut completion_words = captures_after.chain(captures_before).collect::<Vec<_>>();

    match direction {
        CompletionDirection::Forward => {}
        CompletionDirection::Backward => completion_words.reverse(),
    }

    completion_words
        .into_iter()
        .unique()
        .filter(|word| !word.is_empty())
        .collect()
}

impl Editor {
    /// Get all the words that start with starts
    /// Excpects that start only contains word characters
    fn get_completion_matches_forward(&self) -> Vec<&str> {
        let current_buffer = self.current_buffer();
        let raw_cursor_position = current_buffer.raw_position();
        let content = current_buffer.content.inner();
        get_completion_matches(content, raw_cursor_position, CompletionDirection::Forward)
    }

    fn get_completion_matches_backward(&self) -> Vec<&str> {
        let current_buffer = self.current_buffer();
        let raw_cursor_position = current_buffer.raw_position();
        let content = current_buffer.content.inner();
        get_completion_matches(content, raw_cursor_position, CompletionDirection::Backward)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_test() {
        // Cursor is here          v
        let content = r#"con,cont,cconten content; c_onte' ca""#;
        let completion_matches_backward =
            get_completion_matches(content, 10, CompletionDirection::Backward);

        assert_eq!(
            vec!["cont", "con", "ca", "c_onte", "content", "conten"],
            completion_matches_backward
        );

        let completion_matches_forward =
            get_completion_matches(content, 10, CompletionDirection::Forward);

        assert_eq!(
            vec!["conten", "content", "c_onte", "ca", "con", "cont"],
            completion_matches_forward
        );
    }

    #[test]
    fn dedup_test() {
        // Cursor is here          v
        let content = r#"con, con con"#;
        let completion_matches_backward =
            get_completion_matches(content, 10, CompletionDirection::Forward);

        assert_eq!(vec!["con"], completion_matches_backward);
    }

    #[test]
    fn complete_everything_test() {
        let content = "a, b c d e ";
        let completion_matches_backward =
            get_completion_matches(content, 11, CompletionDirection::Forward);

        assert_eq!(vec!["a", "b", "c", "d", "e"], completion_matches_backward);
    }
}
