use amanita::buffer::Buffer;
use amanita::input::handle_input;
use amanita::ui::Screen;
use crossterm::cursor;
use crossterm::QueueableCommand;
use std::io::stdout;
use std::io::Read;
use std::io::Write;

use anyhow::Result;
use std::io;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let mut screen = Screen::new()?;
    let file = Path::new("src/buffer.rs");

    let mut buffer = Buffer::from_file(file).await;

    stdout().queue(cursor::MoveTo(0, 0))?.flush()?;
    buffer.render(&mut screen)?;

    handle_input(&mut buffer, &mut screen).await?;
    // sleep(Duration::from_millis(2000)).await;

    // let mut in_buffer = [0; 1];
    // let mut reader = io::stdin();
    // reader.read_exact(&mut in_buffer)?;
    // let in_buffer = std::str::from_utf8(&in_buffer)?;
    // println!("{in_buffer}");

    // sleep(Duration::from_millis(20000)).await;

    // let file = Path::new("src/ui.rs");
    // let buffer = Buffer::from_file(file).await;
    // buffer.render(&mut screen)?;
    // sleep(Duration::from_millis(2000)).await;

    Ok(())
}
