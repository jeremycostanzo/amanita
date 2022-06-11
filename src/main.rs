use amanita::buffer::Buffer;
use amanita::input::handle_input;
use amanita::EditorBuilder;
use crossterm::cursor;
use crossterm::QueueableCommand;
use std::env;
use std::io::stdout;

use std::io::Write;

use anyhow::Result;

use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];

    let file_path = Path::new(&file_name);
    let buffers = vec![Buffer::from_file(file_path).await];

    let mut editor = EditorBuilder::new()
        .buffers(buffers)
        .line_wrap(false)
        .build();

    stdout().queue(cursor::MoveTo(0, 0))?.flush()?;
    editor.render()?;

    handle_input(&mut editor).await?;

    Ok(())
}
