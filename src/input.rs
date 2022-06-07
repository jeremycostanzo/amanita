use crate::buffer::Buffer;
use crate::movement::Direction;
use crate::ui::Screen;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use futures::{future::FutureExt, StreamExt};

use anyhow::Result;

use crossterm::event::{Event, EventStream};

pub async fn handle_input(buffer: &mut Buffer, screen: &mut Screen) -> Result<()> {
    let mut reader = EventStream::new();

    loop {
        let event = reader.next().fuse();

        match event.await {
            Some(Ok(event)) => {
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::CONTROL,
                    }) => {
                        buffer.save().await?;
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                    }) => {
                        buffer.insert_char(c, screen);
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::SHIFT,
                    }) => {
                        let uppercase_chars = c.to_uppercase().collect::<Vec<_>>();
                        if uppercase_chars.len() == 1 {
                            buffer.insert_char(uppercase_chars[0], screen);
                        }
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::CONTROL,
                    }) => {
                        buffer.move_to_next_word(screen);
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        ..
                    }) => {
                        buffer.move_cursor(Direction::Right, 1, screen);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Up, ..
                    }) => {
                        buffer.move_cursor(Direction::Up, 1, screen);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        ..
                    }) => {
                        buffer.move_cursor(Direction::Left, 1, screen);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        ..
                    }) => {
                        buffer.move_cursor(Direction::Down, 1, screen);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    }) => {
                        buffer.delete_char(screen);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Tab, ..
                    }) => {
                        for _ in 0..4 {
                            buffer.insert_char('\t', screen);
                        }
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }) => {
                        buffer.insert_newline(screen);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    }) => {
                        break;
                    }
                    _ => {}
                };
                buffer.render(screen)?;
            }
            Some(Err(e)) => println!("Error: {:?}\r", e),
            None => continue,
        }
    }
    Ok(())
}
