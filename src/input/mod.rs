mod insert;
mod normal;
mod visual;

use crate::editor::Editor;
use crate::modes::Mode;

use futures::{future::FutureExt, StreamExt};

use anyhow::Result;

use crossterm::event::EventStream;

pub struct LeaveProgram;

pub async fn handle_input(editor: &mut Editor) -> Result<()> {
    let mut reader = EventStream::new();

    loop {
        let event = reader.next().fuse();

        match event.await {
            Some(Ok(event)) => {
                let mode = &editor.mode;
                let leave_program = match mode {
                    Mode::Insert => insert::handle_event(event, editor).await,
                    Mode::Normal => normal::handle_event(event, editor).await,
                    Mode::Visual => visual::handle_event(event, editor).await,
                }?;

                if let Some(LeaveProgram) = leave_program {
                    break;
                }

                editor.render()?;
            }
            Some(Err(e)) => println!("Error: {:?}\r", e),
            None => continue,
        }
    }
    Ok(())
}
