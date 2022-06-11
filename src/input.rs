use crate::movement::Movement;
use crate::Editor;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use futures::{future::FutureExt, StreamExt};

use anyhow::Result;

use crossterm::event::{Event, EventStream};

pub async fn handle_input(editor: &mut Editor) -> Result<()> {
    let mut reader = EventStream::new();

    loop {
        let event = reader.next().fuse();

        match event.await {
            Some(Ok(event)) => {
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::CONTROL,
                    }) => {
                        editor.save().await?;
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                    }) => {
                        editor.insert_char(c);
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::SHIFT,
                    }) => {
                        let uppercase_chars = c.to_uppercase().collect::<Vec<_>>();
                        if uppercase_chars.len() == 1 {
                            editor.insert_char(uppercase_chars[0]);
                        }
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::CONTROL,
                    }) => {
                        Movement::Word(1).do_move(editor);
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        ..
                    }) => {
                        Movement::Cursor(1).do_move(editor);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Up, ..
                    }) => {
                        Movement::Line(-1).do_move(editor);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::CONTROL,
                    }) => {
                        Movement::Word(-1).do_move(editor);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        ..
                    }) => {
                        Movement::Cursor(-1).do_move(editor);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        ..
                    }) => {
                        Movement::Line(1).do_move(editor);
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    }) => {
                        editor.delete_char();
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Tab, ..
                    }) => {
                        for _ in 0..4 {
                            editor.insert_char('\t');
                        }
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }) => {
                        editor.insert_newline();
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    }) => {
                        break;
                    }
                    _ => {}
                };
                editor.render()?;
            }
            Some(Err(e)) => println!("Error: {:?}\r", e),
            None => continue,
        }
    }
    Ok(())
}
