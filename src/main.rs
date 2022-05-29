use amanita::buffer::Buffer;
use amanita::ui::Screen;

use anyhow::Result;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let mut screen = Screen::new()?;
    let file = Path::new("src/buffer.rs");
    let buffer = Buffer::from_file(file).await;
    buffer.render(&mut screen)?;
    sleep(Duration::from_millis(2000)).await;

    let file = Path::new("src/ui.rs");
    let buffer = Buffer::from_file(file).await;
    buffer.render(&mut screen)?;
    sleep(Duration::from_millis(2000)).await;

    Ok(())
}
