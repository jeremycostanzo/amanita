use crate::actions::Movement;
use crate::editor::Editor;
use crate::modes::Mode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::LeaveProgram;
use crossterm::event::Event;

pub async fn handle_event(
    event: Event,
    editor: &mut Editor,
) -> anyhow::Result<Option<LeaveProgram>> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::CONTROL,
        }) => {
            Movement::Word(1).delete(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Word(1).delete(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::WordEnd(1).delete(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('E'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            Movement::WordEnd(-1).delete(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            ..
        }) => {
            Movement::Cursor(1).delete(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Cursor(1).delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up, ..
        }) => {
            Movement::Line(-1).delete(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Line(-1).delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::CONTROL,
        }) => {
            Movement::Word(-1).delete(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Word(-1).delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Left,
            ..
        }) => {
            Movement::Cursor(-1).delete(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Cursor(-1).delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Down,
            ..
        }) => {
            Movement::Line(1).delete(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Line(1).delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('L'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            Movement::EndOfLine.delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('H'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            Movement::FirstNonWhitespaceOfLine.delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Char {
                char: 'c',
                delta: 0,
            }
            .delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('F'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            Movement::Char {
                char: 'c',
                delta: -1,
            }
            .delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::BeforeChar {
                char: 'c',
                delta: 0,
            }
            .delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('T'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            Movement::BeforeChar {
                char: 'c',
                delta: -1,
            }
            .delete(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::BeginningOfFile.delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('G'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            Movement::EndOfFile.delete(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        }) => {
            return Ok(Some(LeaveProgram));
        }
        _ => {}
    };
    editor.mode = Mode::Normal;

    Ok(None)
}
