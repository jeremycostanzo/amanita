use amanita::buffer::Buffer;
use amanita::input::handle_input;
use amanita::ui::Screen;
use crossterm::cursor;
use crossterm::QueueableCommand;
use std::env;
use std::io::stdout;

use std::io::Write;

use anyhow::Result;

use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let mut screen = Screen::new()?;
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];

    let file_path = Path::new(&file_name);

    let mut buffer = Buffer::from_file(file_path).await;

    stdout().queue(cursor::MoveTo(0, 0))?.flush()?;
    buffer.render(&mut screen)?;

    handle_input(&mut buffer, &mut screen).await?;

    Ok(())
}
