use crate::buffer::Buffer;
use crate::ui::Screen;
use crossterm::event::KeyEvent;
use crossterm::QueueableCommand;
use crossterm::{
    cursor::{self},
};
use std::io::{stdout};


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
                    modifiers: _,
                }) = event
                {
                    buffer.insert(c)?;
                    stdout().queue(cursor::MoveRight(1))?;
                    buffer.render(screen)?;
                }

                if event == Event::Key(KeyCode::Right.into()) {
                    // screen.cursor_forward()?;
                }

                if event == Event::Key(KeyCode::Up.into()) {
                    // cursor_position.y -= 1;
                    // screen
                    //     .terminal
                    //     .queue(cursor::MoveTo(cursor_position.x, cursor_position.y))?;
                    // screen.terminal.flush()?;
                }

                if event == Event::Key(KeyCode::Left.into()) {
                    // screen.cursor_backwards()?;
                }

                if event == Event::Key(KeyCode::Down.into()) {
                    // cursor_position.y += 1;
                    // screen
                    //     .terminal
                    //     .queue(cursor::MoveTo(cursor_position.x, cursor_position.y))?;
                    // screen.terminal.flush()?;
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
