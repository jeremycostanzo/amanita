use amanita::buffer::Buffer;
use amanita::input::handle_input;
use amanita::Editor;
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

    let mut editor: Editor = Default::default();
    editor.with_line_wrap(false).with_buffers(buffers);

    stdout().queue(cursor::MoveTo(0, 0))?.flush()?;
    editor.render()?;

    handle_input(&mut editor).await?;

    Ok(())
}
