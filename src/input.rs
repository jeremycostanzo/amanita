use crate::buffer::{Buffer, Direction};
use crate::ui::Screen;
use crossterm::cursor;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::QueueableCommand;
use std::io::stdout;

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
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                    }) => {
                        buffer.insert(c, screen);
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
                        code: KeyCode::Esc, ..
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
