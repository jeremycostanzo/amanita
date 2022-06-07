use crate::buffer::Buffer;
use crate::ui::Screen;

impl Buffer {
    // Used after a move of cursor, to ensure that the cursor never goes out of a line
    fn adjust_x(&mut self, screen: &Screen) {
        let new_line_size = self.content.inner().lines().nth(self.y()).unwrap().len();
        if self.offset.x > new_line_size {
            self.offset.x = new_line_size.saturating_sub(screen.width as usize);
        }
        if self.x() > new_line_size {
            self.screen_cursor_position.x =
                (new_line_size - self.offset.x as usize).try_into().unwrap();
        }
    }

    fn sub_any(cursor_position: &mut u16, offset: &mut usize, value: usize) {
        let to_remove_to_cursor_position: u16 =
            value.min(*cursor_position as usize).try_into().unwrap();
        let to_remove_to_offset = value - to_remove_to_cursor_position as usize;

        *cursor_position -= to_remove_to_cursor_position;
        *offset = (*offset).saturating_sub(to_remove_to_offset);
    }

    fn sub_x(&mut self, x: usize, screen: &Screen) {
        Buffer::sub_any(&mut self.screen_cursor_position.x, &mut self.offset.x, x);
        self.adjust_x(screen);
    }

    fn sub_y(&mut self, y: usize, screen: &Screen) {
        Buffer::sub_any(&mut self.screen_cursor_position.y, &mut self.offset.y, y);
        self.adjust_x(screen);
    }

    fn add_any(cursor_position: &mut u16, offset: &mut usize, box_size: u16, value: usize) {
        let to_add_to_cursor_position: u16 = (value
            .min((box_size - *cursor_position - 1) as usize))
        .try_into()
        .unwrap();
        let to_add_to_offset = value - to_add_to_cursor_position as usize;

        *cursor_position += to_add_to_cursor_position;
        *offset += to_add_to_offset;
    }

    fn add_x(&mut self, x: usize, screen: &Screen) {
        Buffer::add_any(
            &mut self.screen_cursor_position.x,
            &mut self.offset.x,
            screen.width,
            x,
        );
        self.adjust_x(screen);
    }

    fn add_y(&mut self, y: usize, screen: &Screen) {
        let to_add = y.min(self.content.inner().lines().count() - self.y() - 1);
        Buffer::add_any(
            &mut self.screen_cursor_position.y,
            &mut self.offset.y,
            screen.heigth,
            to_add,
        );
        self.adjust_x(screen);
    }

    pub fn move_cursor(&mut self, direction: Direction, value: usize, screen: &Screen) {
        use Direction::*;
        match direction {
            Up => self.sub_y(value, screen),
            Left => self.sub_x(value, screen),
            Down => self.add_y(value, screen),
            Right => self.add_x(value, screen),
        };
    }

    pub fn move_to_next_word(&mut self, screen: &Screen) {
        let target = self.next_word_index();
        self.move_cursor(Direction::Right, target - self.raw_position(), screen);
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

        if self.screen_cursor_position.x == screen.width {
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
            self.move_cursor(Direction::Right, len, screen);
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
