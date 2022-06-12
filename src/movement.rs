use crate::editor::Editor;
use crate::modes::Mode;

#[derive(Clone, Copy, Debug)]
pub enum Movement {
    // Most basic movement: move the cursor by n characters in the line
    Cursor(i64),
    // Move n lines in the buffer
    Line(i64),
    // Move n words
    Word(i64),
}

impl Movement {
    pub fn do_move(self, editor: &mut Editor) {
        match self {
            Movement::Cursor(delta) => {
                let line_len = editor.current_buffer().current_line().len() as i64;
                let width = editor.screen().width;

                let buffer = editor.current_buffer_mut();
                let position = buffer.x() as i64;
                let target = (position + delta).max(0).min(line_len);

                let boxed_delta = target - position;

                let cursor_position = buffer.screen_cursor_position.x;

                let cursor_position_delta = boxed_delta
                    .max(-(cursor_position as i64))
                    .min((width - cursor_position - 1) as i64);

                let offset_delta = boxed_delta - cursor_position_delta;

                buffer.screen_cursor_position.x =
                    (buffer.screen_cursor_position.x as i64 + cursor_position_delta) as u16;
                buffer.offset.x = ((buffer.offset.x as i64) + offset_delta) as usize;
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
                editor.adjust_x();
            }

            Movement::Word(delta) => {
                let buffer = editor.current_buffer();
                let target = buffer.nth_word_index(delta);
                let cursor_delta = target as i64 - buffer.raw_position() as i64;
                Movement::Cursor(cursor_delta).do_move(editor);
            }
        }
    }
    pub fn visual_move(self, editor: &mut Editor) {
        if editor.mode != Mode::Visual {
            unreachable!()
        }
        self.do_move(editor);
        let new_raw_cursor_position = editor.current_buffer().raw_position();
        let mut last_selection = &mut editor.last_selection;
        last_selection.end = new_raw_cursor_position;
    }
}

impl Editor {
    // Used after a move of cursor, to ensure that the cursor never goes out of a line
    fn adjust_x(&mut self) {
        let width = self.screen().width;
        let buffer = self.current_buffer_mut();
        let new_line_size = buffer.current_line().len();
        if buffer.offset.x > new_line_size {
            buffer.offset.x = new_line_size.saturating_sub(width as usize);
        }
        if buffer.x() > new_line_size {
            buffer.screen_cursor_position.x = (new_line_size - buffer.offset.x as usize)
                .try_into()
                .unwrap();
        }
    }

    pub fn insert_newline(&mut self) {
        let buffer = self.current_buffer();
        let pos = buffer.raw_position();

        Movement::Line(1).do_move(self);

        let buffer = self.current_buffer_mut();
        let content = buffer.content.inner_mut();
        content.insert(pos, '\n');
        buffer.offset.x = 0;
        buffer.screen_cursor_position.x = 0;
    }

    pub fn insert_char(&mut self, c: char) {
        let buffer = self.current_buffer_mut();
        let pos = buffer.raw_position();
        let content = buffer.content.inner_mut();

        content.insert(pos, c);

        Movement::Cursor(1).do_move(self);
    }

    pub fn delete_char(&mut self) {
        let buffer = self.current_buffer_mut();
        let pos = buffer.raw_position();
        if pos == 0 {
            return;
        }

        if buffer.x() == 0 {
            let y = buffer.y();
            let len = buffer.content.inner().lines().nth(y - 1).unwrap().len();

            let content = buffer.content.inner_mut();
            content.remove(pos - 1);
            Movement::Line(-1).do_move(self);
            Movement::Cursor(len as i64).do_move(self);
        } else {
            let content = buffer.content.inner_mut();
            let char = content.remove(pos - 1);
            if char == '\t' {
                content.remove(pos - 2);
                content.remove(pos - 3);
                content.remove(pos - 4);
                Movement::Cursor(-4).do_move(self);
            } else {
                Movement::Cursor(-1).do_move(self);
            }
        }
    }
}
