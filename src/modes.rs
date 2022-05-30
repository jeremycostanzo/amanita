#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum Mode {
    Insert,
    Normal,
    Visual,
}
