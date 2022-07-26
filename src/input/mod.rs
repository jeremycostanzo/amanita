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

use crate::Direction;
use crossterm::event::EventStream;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

impl<'a> Default for MappingConfiguration<'a> {
    fn default() -> Self {
        Self {
            action_mapping: [
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::CONTROL,
                    })],
                    vec![Action::Save],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Char('j'),
                        modifiers: KeyModifiers::CONTROL,
                    })],
                    vec![Action::Complete(&Direction::Backward)],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Char('k'),
                        modifiers: KeyModifiers::CONTROL,
                    })],
                    vec![Action::Complete(&Direction::Forward)],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Char('a'),
                        modifiers: KeyModifiers::CONTROL,
                    })],
                    vec![Action::Move(&Movement::BeginningOfLine)],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Char('e'),
                        modifiers: KeyModifiers::CONTROL,
                    })],
                    vec![Action::Move(&Movement::EndOfLine)],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::CONTROL,
                    })],
                    vec![Action::Move(&Movement::Word(1))],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                    })],
                    vec![Action::Move(&Movement::Cursor(1))],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                    })],
                    vec![Action::Move(&Movement::Line(-1))],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::CONTROL,
                    })],
                    vec![Action::Move(&Movement::Word(-1))],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    })],
                    vec![Action::Move(&Movement::Cursor(-1))],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                    })],
                    vec![Action::Move(&Movement::Line(1))],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: KeyModifiers::NONE,
                    })],
                    vec![Action::Delete(&Movement::Cursor(-1))],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Tab,
                        modifiers: KeyModifiers::NONE,
                    })],
                    vec![Action::Insert("\t\t\t\t")],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                    })],
                    vec![Action::Insert("\n")],
                ),
                (
                    vec![Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    })],
                    vec![Action::LeaveProgram],
                ),
            ]
            .into(),
            operator_mapping: [
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('y'),
                        modifiers: KeyModifiers::NONE,
                    }),
                    Operator::Yank,
                ),
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::NONE,
                    }),
                    Operator::Delete,
                ),
            ]
            .into(),
        }
    }
}

use crate::actions::LeaveProgram;

pub async fn handle_input<'a>(
    editor: &mut Editor,
    mapping_configuration: &MappingConfiguration<'a>,
) {
    let mut reader = EventStream::new();

    'outer: loop {
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
                                Ok(Some(LeaveProgram)) => break 'outer,
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

                // If character and insert mode
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                    }) => {
                        editor
                            .insert(c.to_string().as_str())
                            .map_err(|error| error!(?error))
                            .ok();

                        editor.current_chord = Vec::new();
                        continue;
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::SHIFT,
                    }) => {
                        let uppercase_chars = c.to_uppercase().collect::<Vec<_>>();
                        editor
                            .insert_char(uppercase_chars[0])
                            .map_err(|error| error!(?error))
                            .ok();

                        editor.current_chord = Vec::new();
                        continue;
                    }
                    _ => (),
                };

                // Not found
                editor.current_chord = Vec::new();
            }
            None => error!("Input is none"),
            Some(Err(error)) => error!(%error, "Couldn't read input"),
        }
    }
}
