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
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
        }) => {
            editor.save().await?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::CONTROL,
        }) => editor.insert_completion_backward()?,

        Event::Key(KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::CONTROL,
        }) => editor.insert_completion_forward()?,

        Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
        }) => Movement::BeginningOfLine.perform(editor)?,

        Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
        }) => Movement::EndOfLine.perform(editor)?,

        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
        }) => {
            editor.insert_char(c)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            let uppercase_chars = c.to_uppercase().collect::<Vec<_>>();
            if uppercase_chars.len() == 1 {
                editor.insert_char(uppercase_chars[0])?;
            }
        }

        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::CONTROL,
        }) => {
            Movement::Word(1).perform(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Right,
            ..
        }) => {
            Movement::Cursor(1).perform(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up, ..
        }) => {
            Movement::Line(-1).perform(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::CONTROL,
        }) => {
            Movement::Word(-1).perform(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Left,
            ..
        }) => {
            Movement::Cursor(-1).perform(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Down,
            ..
        }) => {
            Movement::Line(1).perform(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            ..
        }) => {
            editor.delete_char()?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Tab, ..
        }) => {
            for _ in 0..4 {
                editor.insert_char('\t')?;
            }
        }

        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            ..
        }) => {
            editor.insert_newline()?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
        }) => {
            editor.leave_insert_mode()?;
            editor.mode = Mode::Normal;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        }) => {
            return Ok(Some(LeaveProgram));
        }
        _ => {}
    };

    tracing::info!("completion: {:?}", &editor.completion_words);

    if !matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Char('p' | 'n'),
            modifiers: KeyModifiers::CONTROL,
        })
    ) {
        editor.completion_words = None
    }

    Ok(None)
}
