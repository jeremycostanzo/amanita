use crate::buffer::Cell;

use crate::buffer::Buffer;
use crate::ui::Screen;

impl Buffer {
    // Used after a move of cursor, to ensure that the cursor never goes out of a line
    fn adjust_x(&mut self, screen: &Screen) {
        let new_line_size = self.content.inner().get(self.y()).unwrap().len();
        if self.offset.x > new_line_size {
            self.offset.x = new_line_size.saturating_sub(screen.width as usize);
        }
        if self.x() > new_line_size {
            self.cursor_position.x = (new_line_size - self.offset.x as usize).try_into().unwrap();
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
        Buffer::sub_any(&mut self.cursor_position.x, &mut self.offset.x, x);
        self.adjust_x(screen);
    }

    fn sub_y(&mut self, y: usize, screen: &Screen) {
        Buffer::sub_any(&mut self.cursor_position.y, &mut self.offset.y, y);
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
            &mut self.cursor_position.x,
            &mut self.offset.x,
            screen.width,
            x,
        );
        self.adjust_x(screen);
    }

    fn add_y(&mut self, y: usize, screen: &Screen) {
        let to_add = y.min(self.content.inner().len() - self.y() - 1);
        Buffer::add_any(
            &mut self.cursor_position.y,
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

    pub fn insert(&mut self, c: char, screen: &Screen) {
        let (x, y) = (self.x(), self.y());
        let content = self.content.inner_mut();
        let line = content.get_mut(y);
        let line = match line {
            Some(line) => line,
            None => match content.last_mut() {
                Some(line) => line,
                None => {
                    content.push(Vec::new());
                    content.get_mut(0).unwrap()
                }
            },
        };

        line.insert(x, Cell { symbol: c });

        if self.cursor_position.x == screen.width {
            self.offset.x += 1;
        } else {
            self.cursor_position.x += 1;
        }
    }

    pub fn delete_char(&mut self, screen: &Screen) {
        let x = self.x();
        let y = self.y();
        let inner = self.content.inner_mut();
        if x == 0 {
            if y != 0 {
                let current_line = inner.get(y).unwrap().clone();
                let previous_line = inner.get_mut(y - 1).unwrap();
                let previous_line_length = previous_line.len();
                previous_line.extend(current_line);
                inner.remove(y);
                self.move_cursor(Direction::Up, 1, screen);
                self.move_cursor(Direction::Right, previous_line_length, screen);
            }
        } else {
            let line = inner.get_mut(y).unwrap();
            let char = line.get(x - 1).unwrap().symbol;
            if char == '\t' {
                for i in 1..5 {
                    line.remove((x) - i);
                }
                self.move_cursor(Direction::Left, 4, screen);
            } else {
                line.remove((x) - 1);
                self.move_cursor(Direction::Left, 1, screen);
            }
        }
        self.adjust_x(screen);
    }

    pub fn add_new_line(&mut self, screen: &Screen) {
        let y = self.y();
        let x = self.x();
        let inner = self.content.inner_mut();
        let current_line = inner.get_mut(y).unwrap();
        let to_add = current_line.drain(x..).collect();
        inner.insert(y + 1, to_add);
        self.move_cursor(Direction::Down, 1, screen);
        self.offset.x = 0;
        self.cursor_position.x = 0;
    }
}

pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}
