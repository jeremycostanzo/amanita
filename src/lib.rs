pub mod buffer;
pub mod editor;
pub mod input;
pub mod modes;
pub mod movement;
pub mod ui;

pub use editor::EditorBuilder;

#[derive(Debug)]
struct OutOfBounds(usize);
impl std::fmt::Display for OutOfBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Accessed out of bounds index {}", self.0)
    }
}

impl std::error::Error for OutOfBounds {}
