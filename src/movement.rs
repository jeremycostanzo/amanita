use crate::editor::Editor;
use crate::modes::Mode;
use anyhow::Context;
use anyhow::{anyhow, bail, Result};

#[derive(Clone, Copy, Debug)]
pub enum Movement {
    // Most basic movement: move the cursor by n characters in the line
    Cursor(i64),
    // Move n lines in the buffer
    Line(i64),
    // Move n words
    Word(i64),
    // Move the cursor by n characters in the buffer
    CursorUnbounded(i64),
    // Go to
    ToRaw(usize),

    EndOfLine,
    BeginningOfLine,
    FirstNonWhitespaceOfLine,
    Char { char: char, delta: i64 },
}

impl Movement {
    pub fn do_move(self, editor: &mut Editor) -> Result<()> {
        match self {
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
            Movement::CursorUnbounded(delta) => {
                let buffer = editor.current_buffer();
                let content = buffer.content.inner();

                let current_position = editor.current_buffer().raw_position();

                let target = (current_position as i64 + delta).max(0) as usize;

                let min = target.min(current_position);
                let max = (target.max(current_position) as usize).min(content.len());

                let max_is_new_line = content[max..].starts_with('\n');

                let bounded_content = &content[min..(max + 1)];

                let new_lines = bounded_content.matches('\n').count();

                let absolute_lines_delta = if max_is_new_line {
                    new_lines.saturating_sub(1)
                } else {
                    new_lines
                };

                let lines_delta = if current_position > target {
                    -(absolute_lines_delta as i64)
                } else {
                    absolute_lines_delta as i64
                };

                Movement::Line(lines_delta).do_move(editor)?;

                let current_position = editor.current_buffer().raw_position() as i64;
                let cursor_delta = target as i64 - current_position;

                Movement::Cursor(cursor_delta).do_move(editor)?;
                Ok(())
            }
            Movement::ToRaw(raw_position) => {
                let current_raw_position = editor.current_buffer().raw_position() as i64;
                let delta = raw_position as i64 - current_raw_position;
                Movement::CursorUnbounded(delta).do_move(editor)?;
                Ok(())
            }
            Movement::Line(delta) => {
                let heigth = editor.screen().heigth;

                let buffer = editor.current_buffer_mut();

                let y = buffer.y() as i64;
                let boxed_delta = delta
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

            Movement::Word(delta) => {
                let buffer = editor.current_buffer();
                let target = buffer.nth_word_index(delta);
                let cursor_delta = target as i64 - buffer.raw_position() as i64;
                Movement::CursorUnbounded(cursor_delta).do_move(editor)?;
                Ok(())
            }
            Movement::EndOfLine => {
                let current_mode = editor.mode.clone();
                let current_buffer = editor.current_buffer();
                let line_length = current_buffer.current_line_length()?;
                let target = if current_mode == Mode::Insert {
                    line_length
                } else {
                    line_length.saturating_sub(1)
                };
                let x = current_buffer.x();
                let delta = target.checked_sub(current_buffer.x()).ok_or_else(|| {
                    anyhow!("attempted to substract {} from {}", x, target).context("End of line")
                })?;
                Movement::Cursor(delta as i64).do_move(editor)
            }
            Movement::BeginningOfLine => {
                let current_buffer = editor.current_buffer();
                let x = current_buffer.x();
                Movement::Cursor(-(x as i64)).do_move(editor)
            }
            Movement::Char { char, delta } => {
                let current_buffer = editor.current_buffer();
                let content = current_buffer.content.inner();
                let current_position = current_buffer.raw_position();
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

                if let Some(target) = target {
                    Movement::ToRaw(target).do_move(editor)?;
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
                Movement::Cursor(x_to - (x as i64)).do_move(editor)
            }
        }
    }

    pub fn visual_move(self, editor: &mut Editor) -> Result<()> {
        if editor.mode != Mode::Visual {
            bail!("Editor mode is {} but visual move was called", editor.mode);
        }
        self.do_move(editor).with_context(|| "Visual move")?;
        let new_raw_cursor_position = editor.current_buffer().raw_position();
        let mut last_selection = &mut editor.last_selection;
        last_selection.end = new_raw_cursor_position;
        Ok(())
    }

    pub fn delete(self, editor: &mut Editor) -> Result<()> {
        let position = editor.current_buffer().raw_position();
        self.do_move(editor).context("Delete")?;
        let position_after_move = editor.current_buffer().raw_position();

        let from = position.min(position_after_move);
        let to = if position > position_after_move {
            position
        } else {
            position_after_move + 1
        };

        let len = editor.current_buffer().content.inner().len();
        let boxed_to = to.min(len - 1);
        editor
            .current_buffer_mut()
            .content
            .inner_mut()
            .replace_range(from..boxed_to, "");

        editor.adjust_y()?;
        editor.adjust_x()?;
        Movement::ToRaw(from).do_move(editor)?;
        Ok(())
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

        Movement::Line(1).do_move(self).context("Insert new line")?;

        let buffer = self.current_buffer_mut();
        let content = buffer.content.inner_mut();
        content.insert(pos, '\n');
        buffer.offset.x = 0;
        buffer.screen_cursor_position.x = 0;
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
        Movement::ToRaw(indice + 1).do_move(self)?;
        self.insert_newline()?;
        Movement::Line(-1).do_move(self)?;
        Ok(())
    }

    pub fn insert_char(&mut self, c: char) -> Result<()> {
        let buffer = self.current_buffer_mut();
        let pos = buffer.raw_position();
        let content = buffer.content.inner_mut();

        content.insert(pos, c);

        Movement::Cursor(1).do_move(self).map_err(Into::into)
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
            Movement::Line(-1).do_move(self)?;
            Movement::Cursor(len as i64).do_move(self)?;
        } else {
            let content = buffer.content.inner_mut();
            let char = content.remove(pos - 1);
            if char == '\t' {
                content.remove(pos - 2);
                content.remove(pos - 3);
                content.remove(pos - 4);
                Movement::Cursor(-4).do_move(self)?;
            } else {
                Movement::Cursor(-1).do_move(self)?;
            }
        }
        Ok(())
    }
}
