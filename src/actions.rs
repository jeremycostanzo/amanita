use crate::editor::{Clipboard, Editor};
use crate::modes::Mode;
use anyhow::Context;
use anyhow::{bail, Result};

type Content = String;
type At = usize;
type From = usize;
type To = usize;

#[derive(Clone, Debug)]
pub enum Action {
    Insert(At, Content),
    Delete(From, To),
}

impl Action {
    /// perform does the action and returns the action that is necessary to undo it
    pub fn perform(&self, editor: &mut Editor) -> Result<Action> {
        use Action::*;
        match self {
            Insert(at, content) => {
                Movement::ToRaw(*at).perform(editor)?;
                editor.insert(content)?;
                let len = content.len();
                Ok(Delete(*at, at + len))
            }
            Delete(from, to) => {
                let content = editor.delete(*from, *to);
                Movement::ToRaw(*from).perform(editor)?;
                Ok(Insert(*from, content))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Movement {
    // Most basic movement: move the cursor by n characters in the line
    Cursor(i64),
    // Move n lines in the buffer
    Line(i64),
    // Move n words
    Word(i64),
    // Move to n word end
    WordEnd(i64),
    // Move the cursor by n characters in the buffer
    CursorUnbounded(i64),
    // Go to
    ToRaw(usize),

    EndOfLine,
    BeginningOfLine,
    FirstNonWhitespaceOfLine,
    Char { char: char, delta: i64 },
    BeforeChar { char: char, delta: i64 },
    BeginningOfFile,
    EndOfFile,
}

impl Movement {
    pub fn perform(&self, editor: &mut Editor) -> Result<()> {
        match self {
            Movement::Line(delta) => {
                let heigth = editor.screen().heigth;

                let buffer = editor.current_buffer_mut();

                let y = buffer.y() as i64;
                let boxed_delta = (*delta)
                    .max(-y)
                    .min(buffer.content.inner().lines().count() as i64 - y - 1);
                let cursor_position = buffer.screen_cursor_position.y;
                let cursor_position_delta = boxed_delta
                    .max(-(cursor_position as i64))
                    .min((heigth - cursor_position - 1) as i64);
                let offset_delta = boxed_delta - cursor_position_delta;

                buffer.screen_cursor_position.y =
                    (buffer.screen_cursor_position.y as i64 + cursor_position_delta) as u16;
                buffer.offset.y = ((buffer.offset.y as i64) + offset_delta) as usize;
                editor.adjust_x().map_err(Into::into)
            }
            Movement::Cursor(delta) => {
                let line_len = editor
                    .current_buffer()
                    .current_line()
                    .with_context(|| format!("Move cursor of {}", delta))?
                    .len();

                let width = editor.screen().width;
                let current_mode = editor.mode.clone();

                let buffer = editor.current_buffer_mut();
                let position = buffer.x() as i64;
                let upper_bound = if current_mode == Mode::Insert {
                    line_len
                } else {
                    line_len.saturating_sub(1)
                } as i64;
                let target = (position + delta).max(0).min(upper_bound);

                let boxed_delta = target - position;

                let cursor_position = buffer.screen_cursor_position.x;

                let cursor_position_delta = boxed_delta
                    .max(-(cursor_position as i64))
                    .min((width - cursor_position - 1) as i64);

                let offset_delta = boxed_delta - cursor_position_delta;

                buffer.screen_cursor_position.x =
                    (buffer.screen_cursor_position.x as i64 + cursor_position_delta) as u16;
                buffer.offset.x = ((buffer.offset.x as i64) + offset_delta) as usize;
                Ok(())
            }
            Movement::ToRaw(target) => {
                let target = *target;
                let current_raw_position = editor.current_buffer().raw_position();
                let content = editor.current_buffer().content.inner();

                let min = target.min(current_raw_position);
                let max = (target.max(current_raw_position) as usize).min(content.len());

                let max_is_new_line = content[max..].starts_with('\n');

                let bounded_content = &content[min..=max];

                let new_lines = bounded_content.matches('\n').count();

                let absolute_lines_delta = if max_is_new_line {
                    new_lines.saturating_sub(1)
                } else {
                    new_lines
                };

                let lines_delta = if current_raw_position > target {
                    -(absolute_lines_delta as i64)
                } else {
                    absolute_lines_delta as i64
                };

                Movement::Line(lines_delta).perform(editor)?;

                let current_position = editor.current_buffer().raw_position() as i64;
                let cursor_delta = target as i64 - current_position;

                Movement::Cursor(cursor_delta).perform(editor)
            }

            Movement::CursorUnbounded(delta) => {
                let current_position = editor.current_buffer().raw_position();
                let target = (current_position as i64 + delta).max(0) as usize;

                Movement::ToRaw(target).perform(editor)
            }

            Movement::WordEnd(delta) => {
                let delta = *delta;
                let buffer = editor.current_buffer();
                let target = buffer.nth_word_end_index(delta);
                Movement::ToRaw(target).perform(editor)?;
                Ok(())
            }

            Movement::Word(delta) => {
                let delta = *delta;
                let buffer = editor.current_buffer();
                let target = buffer.nth_word_index(delta);
                Movement::ToRaw(target).perform(editor)?;
                Ok(())
            }

            Movement::EndOfLine => {
                let current_buffer = editor.current_buffer();
                let len = current_buffer.current_line_length()?;
                Movement::Cursor(len as i64).perform(editor)
            }

            Movement::BeginningOfLine => {
                let current_buffer = editor.current_buffer();
                let x = current_buffer.x();
                Movement::Cursor(-(x as i64)).perform(editor)
            }
            Movement::Char { char, delta } => {
                let current_buffer = editor.current_buffer();
                let target = current_buffer.next_char_index(*char, *delta);

                if let Some(target) = target {
                    Movement::ToRaw(target).perform(editor)?;
                }

                Ok(())
            }

            Movement::BeforeChar { char, delta } => {
                let delta = *delta;
                let current_buffer = editor.current_buffer();
                let target = current_buffer.next_char_index(*char, delta);

                if let Some(target) = target {
                    if delta >= 0 {
                        Movement::ToRaw(target.saturating_sub(1)).perform(editor)?;
                    } else {
                        Movement::ToRaw(target + 1).perform(editor)?;
                    }
                }

                Ok(())
            }

            Movement::FirstNonWhitespaceOfLine => {
                let current_buffer = editor.current_buffer();
                let x = current_buffer.x();
                let line = current_buffer.current_line()?;
                let index = line
                    .chars()
                    .enumerate()
                    .find(|(_, char)| !char.is_whitespace())
                    .map(|(i, _)| i);

                let x_to = index.unwrap_or(0) as i64;
                Movement::Cursor(x_to - (x as i64)).perform(editor)
            }
            Movement::BeginningOfFile => Movement::ToRaw(0).perform(editor),
            Movement::EndOfFile => {
                let len = editor.current_buffer().content.inner().len();
                Movement::ToRaw(len.saturating_sub(1)).perform(editor)
            }
        }
    }

    pub fn visual_move(self, editor: &mut Editor) -> Result<()> {
        if editor.mode != Mode::Visual {
            bail!("Editor mode is {} but visual move was called", editor.mode);
        }
        self.perform(editor).with_context(|| "Visual move")?;
        let new_raw_cursor_position = editor.current_buffer().raw_position();
        let mut last_selection = &mut editor.last_selection;
        last_selection.end = new_raw_cursor_position;
        Ok(())
    }

    pub fn delete(self, editor: &mut Editor) -> Result<()> {
        let old_position = editor.current_buffer().raw_position();
        self.perform(editor).context("Delete")?;
        let position_after_move = editor.current_buffer().raw_position();

        let from = old_position.min(position_after_move);
        let to = if old_position > position_after_move {
            old_position
        } else {
            position_after_move + 1
        };

        let len = editor.current_buffer().content.inner().len();
        let boxed_to = to.min(len - 1);
        editor.clipboard = Clipboard {
            content: editor.current_buffer().content.inner()[from..boxed_to]
                .chars()
                .collect(),
        };

        let deleted_content = editor.current_buffer().content.inner()[from..boxed_to].to_owned();

        editor
            .current_buffer_mut()
            .content
            .inner_mut()
            .replace_range(from..boxed_to, "");

        // In case last line is deleted to prevent the cursor from going out of bounds
        editor.adjust_y()?;
        editor.adjust_x()?;

        Movement::ToRaw(from).perform(editor)?;
        editor.clipboard = Clipboard {
            content: deleted_content.clone(),
        };

        editor.undo_tree.push(Action::Insert(from, deleted_content));
        Ok(())
    }

    pub fn yank(&self, editor: &mut Editor) -> Result<()> {
        let old_position = editor.current_buffer().raw_position();
        self.perform(editor).context("First move in yank")?;
        let new_position = editor.current_buffer().raw_position();
        let min = old_position.min(new_position);
        let max = old_position.max(new_position);
        editor.clipboard = Clipboard {
            content: editor.current_buffer().content.inner()[min..=max]
                .chars()
                .collect(),
        };

        Movement::ToRaw(old_position)
            .perform(editor)
            .context("Move back when yanking")
    }
}

impl Editor {
    // Used after a move of cursor, to ensure that the cursor never goes out of a line
    fn adjust_x(&mut self) -> Result<()> {
        let width = self.screen().width;
        let current_mode = self.mode.clone();
        let buffer = self.current_buffer_mut();
        let new_line_size = buffer.current_line().context("Adjust x")?.len();
        let upper_bound = if current_mode == Mode::Insert {
            new_line_size
        } else {
            new_line_size.saturating_sub(1)
        };
        if buffer.offset.x > upper_bound {
            buffer.offset.x = new_line_size.saturating_sub(width as usize);
        }
        if buffer.x() >= new_line_size {
            buffer.screen_cursor_position.x =
                (upper_bound.saturating_sub(1 + buffer.offset.x as usize)).try_into()?;
        }
        Ok(())
    }

    // Used after a deletion to ensure that the cursor doesn't stay in a line that doesn't exist
    // anymore
    fn adjust_y(&mut self) -> Result<()> {
        let lines_count = self.current_buffer().lines_count().context("Adjust y")?;

        let buffer = self.current_buffer_mut();
        if buffer.offset.y > lines_count {
            buffer.offset.y = lines_count.saturating_sub(1);
            buffer.screen_cursor_position.y = 0;
            return Ok(());
        } else if buffer.y() >= lines_count {
            buffer.screen_cursor_position.y =
                (lines_count.saturating_sub(1 + buffer.offset.y as usize)).try_into()?;
        }
        Ok(())
    }

    pub fn insert_newline(&mut self) -> Result<()> {
        let buffer = self.current_buffer();
        let pos = buffer.raw_position();

        let content = self.current_buffer_mut().content.inner_mut();
        content.insert(pos, '\n');

        Movement::Line(1).perform(self).context("Insert new line")?;

        let buffer = self.current_buffer_mut();
        buffer.offset.x = 0;
        buffer.screen_cursor_position.x = 0;

        let position = self.current_buffer().raw_position();

        self.undo_tree
            .push(Action::Delete(position.saturating_sub(1), position));

        Ok(())
    }

    pub fn insert_newline_in_n_lines(&mut self, n: i64) -> Result<()> {
        let buffer = &mut self.current_buffer_mut();
        let pos = buffer.raw_position();
        let content = buffer.content.inner_mut();
        let indice = if n >= 0 {
            content[pos..]
                .match_indices('\n')
                .nth(n as usize)
                .map(|(indice, _)| indice)
                .unwrap_or(content.len())
                + pos
        } else {
            content[..pos]
                .match_indices('\n')
                .rev()
                .nth((-n - 1) as usize)
                .map(|(indice, _)| indice)
                .unwrap_or(0)
        };
        Movement::ToRaw(indice + 1).perform(self)?;
        self.insert_newline()?;
        Movement::Line(-1).perform(self)?;
        Ok(())
    }

    pub fn insert_char(&mut self, c: char) -> Result<()> {
        self.insert(c.to_string().as_str())
    }

    pub fn insert(&mut self, content: &str) -> Result<()> {
        let pos = self.current_buffer().raw_position();
        self.current_buffer_mut()
            .content
            .inner_mut()
            .insert_str(pos, content);
        let len = content.len();

        Movement::CursorUnbounded(len as i64).perform(self)
    }

    // Delete from min(from, to) to (excluding) max(from, to)
    pub fn delete(&mut self, from: usize, to: usize) -> String {
        let len = self.current_buffer().content.inner().len();

        let min = from.min(to).max(0);
        let max = from.max(to).min(len);

        let content = self.current_buffer().content.inner()[min..max].to_owned();

        self.current_buffer_mut()
            .content
            .inner_mut()
            .replace_range(min..max, "");

        content
    }

    pub fn paste(&mut self) -> Result<()> {
        let content = self.clipboard.content.to_string();
        self.insert(&content)
    }

    pub fn delete_char(&mut self) -> Result<()> {
        let buffer = self.current_buffer_mut();
        let pos = buffer.raw_position();
        if pos == 0 {
            return Ok(());
        }

        if buffer.x() == 0 {
            let y = buffer.y();
            let len = buffer.content.inner().lines().nth(y - 1).unwrap().len();

            let content = buffer.content.inner_mut();
            content.remove(pos - 1);
            Movement::Line(-1).perform(self)?;
            Movement::Cursor(len as i64).perform(self)?;
        } else {
            let content = buffer.content.inner_mut();
            let char = content.remove(pos - 1);
            if char == '\t' {
                content.remove(pos - 2);
                content.remove(pos - 3);
                content.remove(pos - 4);
                Movement::Cursor(-4).perform(self)?;
            } else {
                Movement::Cursor(-1).perform(self)?;
            }
        }
        Ok(())
    }

    pub fn undo(&mut self) -> Result<()> {
        tracing::info!("undoing, undo tree: {:?}", &self.undo_tree);
        let undo_action = self.undo_tree.undo();
        if let Some(action) = undo_action {
            let redo_action = action.perform(self)?;
            self.undo_tree.replace_undo(redo_action);
            tracing::info!("undid, undo tree: {:?}", &self.undo_tree);
        }
        Ok(())
    }

    pub fn redo(&mut self) -> Result<()> {
        tracing::info!("redoing, undo tree: {:?}", &self.undo_tree);
        let redo_action = self.undo_tree.redo();
        if let Some(action) = redo_action {
            let undo_action = action.perform(self)?;
            self.undo_tree.replace_redo(undo_action);
            tracing::info!("redid, undo tree: {:?}", &self.undo_tree);
        }
        Ok(())
    }
}
