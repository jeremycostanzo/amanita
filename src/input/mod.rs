mod insert;
mod normal;
mod normal_delete;
mod normal_yank;
mod visual;

use crate::actions::Movement;
use crate::editor::Editor;
use crate::modes::Mode;

use futures::{future::FutureExt, StreamExt};

use anyhow::Result;
use tracing::error;

use crossterm::event::EventStream;

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

use crate::actions::Action;
use crossterm::event::Event;

pub type ActionMapping<'a> = HashMap<Vec<Event>, Vec<Action<'a>>>;
pub enum Operator {
    Delete,
    Yank,
}

impl Operator {
    pub async fn execute(&self, editor: &mut Editor, movement: &Movement) -> anyhow::Result<()> {
        let action = match self {
            Operator::Delete => Action::Delete(movement),
            Operator::Yank => Action::Yank(movement),
        };
        action.execute(editor).await.map(|_| ())
    }
}

pub type OperatorMapping = HashMap<Event, Operator>;

pub struct MappingConfiguration<'a> {
    action_mapping: ActionMapping<'a>,
    operator_mapping: OperatorMapping,
}

use crate::actions::LeaveProgram;

pub async fn input_handler<'a>(
    editor: &mut Editor,
    mapping_configuration: &MappingConfiguration<'a>,
) {
    let mut reader = EventStream::new();

    loop {
        editor
            .render()
            .map_err(|e| error!(?e, "Couldn't render"))
            .ok();

        let event = reader.next().fuse().await;

        match event {
            Some(Ok(event)) => {
                // handle event
                let chord = &mut editor.current_chord;
                chord.push(event);

                // Perfect action match
                match mapping_configuration.action_mapping.get(chord) {
                    Some(actions) => {
                        for action in actions {
                            match action.execute(editor).await {
                                Ok(Some(LeaveProgram)) => break,
                                Ok(None) => (),
                                Err(error) => error!(?error),
                            }
                        }
                        editor.current_chord = Vec::new();

                        continue;
                    }
                    None => (),
                }

                // Operator + Movement
                if let Some(operator) = mapping_configuration.operator_mapping.get(&event) {
                    match mapping_configuration.action_mapping.get(&chord[1..]) {
                        Some(actions) => {
                            if let [Action::Move(movement)] = actions.as_slice() {
                                operator
                                    .execute(editor, movement)
                                    .await
                                    .map_err(|error| error!(?error))
                                    .ok();

                                editor.current_chord = Vec::new();
                                continue;
                            }
                        }
                        None => {
                            if mapping_configuration.action_mapping.iter().any(
                                |(configured_chord, actions)| {
                                    matches!(actions.as_slice(), [Action::Move(_)])
                                        && configured_chord.starts_with(&chord[1..])
                                },
                            ) {
                                continue;
                            }
                        }
                    }
                };

                // Prefix
                if mapping_configuration
                    .action_mapping
                    .keys()
                    .any(|configured_chord| configured_chord.starts_with(&chord[..]))
                {
                    continue;
                };

                // Not found
                editor.current_chord = Vec::new();
            }
            None => error!("Input is none"),
            Some(Err(error)) => error!(%error, "Couldn't read input"),
        }
    }
}
