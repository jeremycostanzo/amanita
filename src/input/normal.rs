use crate::actions::{Action, Movement};
use crate::modes::Mode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crossterm::event::Event;

pub async fn handle_event(event: Event) -> Vec<Action> {
    use Action::*;
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::NONE,
        }) => vec![Undo],

        Event::Key(KeyEvent {
            code: KeyCode::Char('U'),
            modifiers: KeyModifiers::SHIFT,
        }) => vec![Redo],

        Event::Key(KeyEvent {
            code: KeyCode::Char('i'),
            modifiers: KeyModifiers::NONE,
        }) => vec![ChangeMode(Mode::Insert)],
        Event::Key(KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::NONE,
        }) => {
            vec![
                ChangeMode(Mode::Insert),
                Move(Movement::EndOfLine),
                Insert("\n".to_owned()),
            ]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('O'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            vec![
                ChangeMode(Mode::Insert),
                Move(Movement::BeginningOfLine),
                Insert("\n".to_owned()),
                Move(Movement::Line(-1)),
            ]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            vec![ChangeMode(Mode::Insert), Move(Movement::Cursor(1))]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('A'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            vec![ChangeMode(Mode::Insert), Move(Movement::EndOfLine)]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('I'),
            modifiers: KeyModifiers::SHIFT,
        }) => vec![ChangeMode(Mode::Insert), Move(Movement::BeginningOfLine)],

        Event::Key(
            KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::NONE,
            },
        ) => {
            vec![Move(Movement::Word(1))]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::NONE,
        }) => {
            vec![Move(Movement::WordEnd(1))]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('E'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            vec![Move(Movement::WordEnd(-1))]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            ..
        }) => {
            vec![Move(Movement::Cursor(1))]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::NONE,
        }) => {
            vec![Move(Movement::Cursor(1))]
        }

        Event::Key(KeyEvent {
            code: KeyCode::Up, ..
        }) => {
            vec![Move(Movement::Line(-1))]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
        }) => {
            vec![Move(Movement::Line(-1))]
        }

        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::CONTROL,
        }) => {
            vec![Move(Movement::Word(-1))]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::NONE,
        }) => {
            vec![Move(Movement::Word(-1))]
        }

        Event::Key(KeyEvent {
            code: KeyCode::Left,
            ..
        }) => {
            vec![Move(Movement::Cursor(-1))]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
        }) => {
            vec![Move(Movement::Cursor(-1))]
        }

        Event::Key(KeyEvent {
            code: KeyCode::Down,
            ..
        }) => {
            vec![Move(Movement::Line(1))]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
        }) => {
            vec![Move(Movement::Line(1))]
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('L'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            vec![Move(Movement::EndOfLine)]
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('H'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            vec![Move(Movement::FirstNonWhitespaceOfLine)]
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::NONE,
        }) => {
            vec![Move(Movement::Char {
                char: 'c',
                delta: 0,
            })]
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('F'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            vec![Move(Movement::Char {
                char: 'c',
                delta: -1,
            })]
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::NONE,
        }) => {
            vec![Move(Movement::BeforeChar {
                char: 'c',
                delta: 0,
            })]
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('T'),
            modifiers: KeyModifiers::SHIFT,
        }) => {
            vec![Move(Movement::BeforeChar {
                char: 'c',
                delta: -1,
            })]
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::NONE,
        }) => vec![Move(Movement::BeginningOfFile)],

        Event::Key(KeyEvent {
            code: KeyCode::Char('G'),
            modifiers: KeyModifiers::SHIFT,
        }) => vec![Move(Movement::EndOfFile)],

        Event::Key(KeyEvent {
            code: KeyCode::Char('v'),
            modifiers: KeyModifiers::NONE,
        }) => vec![ChangeMode(Mode::Visual)],

        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
        }) => vec![ChangeMode(Mode::NormalDelete)],

        Event::Key(KeyEvent {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::NONE,
        }) => vec![ChangeMode(Mode::NormalYank)],

        Event::Key(KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::NONE,
        }) => vec![Paste],

        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        }) => vec![LeaveProgram],

        _ => {
            vec![]
        }
    }
}
