mod insert;
mod normal;
mod normal_delete;
mod normal_yank;
mod visual;

use crate::editor::Editor;
use crate::modes::Mode;

use futures::{future::FutureExt, StreamExt};

use anyhow::Result;
use tracing::error;

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
                    Mode::NormalDelete => normal_delete::handle_event(event, editor).await,
                    Mode::NormalYank => normal_yank::handle_event(event, editor).await,
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

pub async fn generic_input(editor: &mut Editor) -> Result<()> {
    let mut reader = EventStream::new();
    let event = reader.next().fuse();

    match event.await {
        Some(Ok(event)) => {
            let chord = &mut editor.current_chord;
            chord.push(event);
            todo!();
        }
        error => error!(?error, "Could not find event when polling"),
    }

    Ok(())
}

use std::collections::HashMap;

use crossterm::event::Event;

struct MappingConfiguration(HashMap<Vec<Event>, Vec<Action>>);
