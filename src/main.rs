use amanita::buffer::Buffer;
use amanita::input::handle_input;
use amanita::EditorBuilder;
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal;
use crossterm::QueueableCommand;
use std::env;
use std::io::stdout;

use std::io::Write;

use anyhow::Result;

use std::path::Path;
fn cleanup_terminal() {
    let mut stdout = stdout();

    execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    execute!(stdout, cursor::Show).unwrap();

    terminal::disable_raw_mode().unwrap();
}

// We need to catch panics since we need to close the UI and cleanup the terminal before logging any
// error messages to the screen.
fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        cleanup_terminal();
        println!("{panic_info}");
    }));
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic_hook();
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];

    let file_path = Path::new(&file_name);
    let buffers = vec![Buffer::from_file(file_path).await?];

    let mut editor = EditorBuilder::new().buffers(buffers).build()?;

    stdout().queue(cursor::MoveTo(0, 0))?.flush()?;
    editor.render()?;

    handle_input(&mut editor).await?;

    Ok(())
}
