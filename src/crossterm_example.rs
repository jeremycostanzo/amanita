use crossterm::{
    cursor,
    style::{self, Color, Stylize},
    terminal::{self, ScrollDown, ScrollUp},
    ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode().ok();

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    for y in 0..40 {
        for x in 0..150 {
            if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
                // in this loop we are more efficient by not flushing the stdout.
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    // .queue(style::SetBackgroundColor(Color::Magenta))?
                    .queue(style::PrintStyledContent("l".magenta()))?;

                // stdout.flush()?;
            }
        }
    }
    stdout.flush()?;
    thread::sleep(Duration::from_millis(500));
    stdout.queue(ScrollDown(2))?;
    stdout.flush()?;
    thread::sleep(Duration::from_millis(500));
    stdout.queue(ScrollUp(3))?;
    stdout.flush()?;

    thread::sleep(Duration::from_millis(500));
    stdout.queue(ScrollDown(1))?;
    stdout.flush()?;
    thread::sleep(Duration::from_millis(500));
    stdout.queue(ScrollDown(1))?;
    stdout.flush()?;
    stdout.queue(ScrollUp(1))?;
    stdout.flush()?;
    terminal::disable_raw_mode().ok();
    // stdout.flush()?;
    Ok(())
}
