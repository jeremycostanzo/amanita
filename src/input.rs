use crate::buffer::{Buffer, Direction};
use crate::ui::Screen;
use crossterm::cursor;
use crossterm::event::{KeyEvent, KeyModifiers};
use crossterm::QueueableCommand;
use std::io::stdout;

use futures::{future::FutureExt, StreamExt};

use anyhow::Result;

use crossterm::event::{Event, EventStream, KeyCode};

pub async fn handle_input(buffer: &mut Buffer, screen: &mut Screen) -> Result<()> {
    let mut reader = EventStream::new();

    loop {
        let event = reader.next().fuse();

        match event.await {
            Some(Ok(event)) => {
                if let Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                }) = event
                {
                    buffer.insert(c, screen);
                    buffer.render(screen)?;
                }

                if event == Event::Key(KeyCode::Right.into()) {
                    buffer.move_cursor(Direction::Right, screen);
                    buffer.render(screen)?;
                }

                if event == Event::Key(KeyCode::Up.into()) {
                    buffer.move_cursor(Direction::Up, screen);
                    buffer.render(screen)?;
                }

                if event == Event::Key(KeyCode::Left.into()) {
                    buffer.move_cursor(Direction::Left, screen);
                    buffer.render(screen)?;
                }

                if event == Event::Key(KeyCode::Down.into()) {
                    buffer.move_cursor(Direction::Down, screen);
                    buffer.render(screen)?;
                }

                if event == Event::Key(KeyCode::Backspace.into()) {
                    buffer.delete_char();
                    buffer.render(screen)?;
                }

                if event == Event::Key(KeyCode::Esc.into()) {
                    break;
                }
            }
            Some(Err(e)) => println!("Error: {:?}\r", e),
            None => continue,
        }
    }
    Ok(())
}
