use crate::editor::{Editor, Selection};
use crate::modes::Mode;
use crate::movement::Movement;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::LeaveProgram;
use crossterm::event::Event;

pub async fn handle_event(
    event: Event,
    editor: &mut Editor,
) -> anyhow::Result<Option<LeaveProgram>> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('i'),
            modifiers: KeyModifiers::NONE,
        }) => {
            editor.mode = Mode::Insert;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::NONE,
        }) => {
            editor.insert_newline_in_n_lines(0)?;
            editor.mode = Mode::Insert;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('O'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            editor.insert_newline_in_n_lines(-1)?;
            editor.mode = Mode::Insert;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            editor.mode = Mode::Insert;
            Movement::Cursor(1).do_move(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('A'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            editor.mode = Mode::Insert;
            Movement::EndOfLine.do_move(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('I'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            editor.mode = Mode::Insert;
            Movement::FirstNonWhitespaceOfLine.do_move(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::CONTROL,
        }) => {
            Movement::Word(1).do_move(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Word(1).do_move(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            ..
        }) => {
            Movement::Cursor(1).do_move(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Cursor(1).do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up, ..
        }) => {
            Movement::Line(-1).do_move(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Line(-1).do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::CONTROL,
        }) => {
            Movement::Word(-1).do_move(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Word(-1).do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Left,
            ..
        }) => {
            Movement::Cursor(-1).do_move(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Cursor(-1).do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Down,
            ..
        }) => {
            Movement::Line(1).do_move(editor)?;
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Line(1).do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('L'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            Movement::EndOfLine.do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('H'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            Movement::FirstNonWhitespaceOfLine.do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::Char {
                char: 'c',
                delta: 0,
            }
            .do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('F'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            Movement::Char {
                char: 'c',
                delta: -1,
            }
            .do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::NONE,
        }) => {
            Movement::BeforeChar {
                char: 'c',
                delta: 0,
            }
            .do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('T'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            Movement::BeforeChar {
                char: 'c',
                delta: -1,
            }
            .do_move(editor)?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('v'),
            modifiers: KeyModifiers::NONE,
        }) => {
            let raw_position = editor.current_buffer().raw_position();
            editor.last_selection = Selection::at_cursor(raw_position);
            editor.mode = Mode::Visual;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        }) => {
            return Ok(Some(LeaveProgram));
        }
        _ => {}
    };
    Ok(None)
}
