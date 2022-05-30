use crate::buffer::Buffer;
use crate::modes::Mode;
use crate::ui::Screen;
use anyhow::Result;
use std::collections::BTreeMap;

pub trait Command {
    fn execute(&self, mode: &mut Mode, screen: &mut Screen, buffer: &mut Buffer) -> Result<()>;
}

pub enum ProcessedSequence {
    Match(Box<dyn Command>),
    Prefix,
    NoMatch,
}

// TODO? implement using an automata
pub trait Keybinds {
    fn process_sequence(&self, sequence: &str, mode: Mode) -> ProcessedSequence;
}

pub struct EnterInsertMode;

impl Command for EnterInsertMode {
    fn execute(&self, mode: &mut Mode, screen: &mut Screen, buffer: &mut Buffer) -> Result<()> {
        *mode = Mode::Insert;
        Ok(())
    }
}

pub struct LeaveInsertMode;
impl Command for LeaveInsertMode {
    fn execute(&self, mode: &mut Mode, screen: &mut Screen, buffer: &mut Buffer) -> Result<()> {
        *mode = Mode::Normal;
        Ok(())
    }
}

pub struct KeybindsBTreeMap(BTreeMap<(String, Mode), Box<dyn Command>>);

// Could be better with a BTreeMap considering that we use starts_with with the keys? Automata will
// be better anyway
impl KeybindsBTreeMap {
    fn new() -> Self {
        let mut map: BTreeMap<(String, Mode), Box<dyn Command>> = BTreeMap::new();
        map.insert(("i".to_owned(), Mode::Normal), Box::new(EnterInsertMode {}));
        map.insert(
            ("<Esc>".to_owned(), Mode::Insert),
            Box::new(LeaveInsertMode {}),
        );
        map.insert(("i".to_owned(), Mode::Normal), Box::new(EnterInsertMode {}));
        Self(map)
    }
}

impl Keybinds for KeybindsBTreeMap {
    fn process_sequence(&self, sequence: &str, mode: Mode) -> ProcessedSequence {
        for ((command_sequence, command_mode), command) in self.0.iter() {
            if command_mode == &mode {
                if command_sequence == sequence {
                    return ProcessedSequence::Match(command.clone());
                }
                if sequence.starts_with(command_sequence) {
                    return ProcessedSequence::Prefix;
                }
            }
        }
        ProcessedSequence::NoMatch
    }
}
