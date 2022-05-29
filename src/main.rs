use amanita::buffer::Buffer;
use amanita::ui::Screen;

use anyhow::Result;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let file = Path::new("src/buffer.rs");
    let buffer = Buffer::from_file(file).await;
    buffer.render(Screen::new()?)?;
    sleep(Duration::from_millis(500)).await;

    Ok(())
}
