use crate::buffer::Buffer;
use crate::ui::Screen;

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
    pub fn do_move(self, buffer: &mut Buffer, screen: &Screen) {
        match self {
            Movement::Cursor(delta) => {
                let position = buffer.x() as i64;
                let target = (position + delta)
                    .max(0)
                    .min(buffer.current_line().len() as i64);

                let boxed_delta = target - position;

                let cursor_position = buffer.screen_cursor_position.x;

                let cursor_position_delta = boxed_delta
                    .max(-(cursor_position as i64))
                    .min((screen.width - cursor_position - 1) as i64);

                let offset_delta = boxed_delta - cursor_position_delta;

                buffer.screen_cursor_position.x =
                    (buffer.screen_cursor_position.x as i64 + cursor_position_delta) as u16;
                buffer.offset.x = ((buffer.offset.x as i64) + offset_delta) as usize;
            }
            Movement::Line(delta) => {
                let y = buffer.y() as i64;
                let boxed_delta = delta
                    .max(-y)
                    .min(buffer.content.inner().lines().count() as i64 - y - 1);
                let cursor_position = buffer.screen_cursor_position.y;
                let cursor_position_delta = boxed_delta
                    .max(-(cursor_position as i64))
                    .min((screen.heigth - cursor_position - 1) as i64);
                let offset_delta = boxed_delta - cursor_position_delta;

                buffer.screen_cursor_position.y =
                    (buffer.screen_cursor_position.y as i64 + cursor_position_delta) as u16;
                buffer.offset.y = ((buffer.offset.y as i64) + offset_delta) as usize;
                buffer.adjust_x(screen);
            }

            Movement::Word(_) => todo!(),
        }
    }
}

impl Buffer {
    // Used after a move of cursor, to ensure that the cursor never goes out of a line
    fn adjust_x(&mut self, screen: &Screen) {
        let new_line_size = self.current_line().len();
        if self.offset.x > new_line_size {
            self.offset.x = new_line_size.saturating_sub(screen.width as usize);
        }
        if self.x() > new_line_size {
            self.screen_cursor_position.x =
                (new_line_size - self.offset.x as usize).try_into().unwrap();
        }
    }

    pub fn move_cursor(&mut self, direction: Direction, value: i64, screen: &Screen) {
        use Direction::*;
        match direction {
            Up => Movement::Line(-(value)).do_move(self, screen),
            Left => Movement::Cursor(-(value)).do_move(self, screen),
            Down => Movement::Line(value).do_move(self, screen),
            Right => Movement::Cursor(value).do_move(self, screen),
        };
    }

    pub fn move_to_next_word(&mut self, screen: &Screen) {
        let target = self.next_word_index();
        self.move_cursor(
            Direction::Right,
            (target - self.raw_position()) as i64,
            screen,
        );
    }

    pub fn move_to_previous_word(&mut self, screen: &Screen) {
        let target = self.previous_word_index();
        self.move_cursor(
            Direction::Left,
            (self.raw_position() - target) as i64,
            screen,
        );
    }

    pub fn insert_newline(&mut self, screen: &Screen) {
        let pos = self.raw_position();
        let content = self.content.inner_mut();
        content.insert(pos, '\n');
        self.move_cursor(Direction::Down, 1, screen);
        self.offset.x = 0;
        self.screen_cursor_position.x = 0;
    }

    pub fn insert_char(&mut self, c: char, screen: &Screen) {
        let pos = self.raw_position();
        let content = self.content.inner_mut();
        content.insert(pos, c);

        if self.screen_cursor_position.x >= screen.width - 1 {
            self.offset.x += 1;
        } else {
            self.screen_cursor_position.x += 1;
        }
    }

    pub fn delete_char(&mut self, screen: &Screen) {
        let pos = self.raw_position();
        if pos == 0 {
            return;
        }

        if self.x() == 0 {
            let y = self.y();
            let len = self.content.inner().lines().nth(y - 1).unwrap().len();

            let content = self.content.inner_mut();
            content.remove(pos - 1);
            self.move_cursor(Direction::Up, 1, screen);
            self.move_cursor(Direction::Right, len as i64, screen);
        } else {
            let content = self.content.inner_mut();
            let char = content.remove(pos - 1);
            if char == '\t' {
                content.remove(pos - 2);
                content.remove(pos - 3);
                content.remove(pos - 4);
                self.move_cursor(Direction::Left, 4, screen);
            } else {
                self.move_cursor(Direction::Left, 1, screen);
            }
        }
    }
}

pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}
