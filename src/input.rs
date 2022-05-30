use crate::buffer::Buffer;
use crate::ui::Screen;
use crossterm::event::KeyEvent;
use crossterm::style::{self, Stylize};
use crossterm::QueueableCommand;
use std::io::Write;
use std::time::Duration;

use futures::{future::FutureExt, StreamExt};
use futures_timer::Delay;

use anyhow::Result;

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode},
};

pub async fn handle_input(buffer: &mut Buffer, screen: &mut Screen) -> Result<()> {
    let mut reader = EventStream::new();

    loop {
        let mut delay = Delay::new(Duration::from_millis(3_000)).fuse();
        let event = reader.next().fuse();

        match event.await {
            Some(Ok(event)) => {
                if let Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: _,
                }) = event
                {
                    buffer.insert(c)?;
                    println!("{c}");
                    // buffer.render(screen)?;
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
