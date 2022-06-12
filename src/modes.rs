#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}
