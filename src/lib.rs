pub mod buffer;
pub mod input;
pub mod movement;
pub mod ui;

use crate::buffer::Buffer;
use crate::ui::Screen;

#[derive(Debug, Default, Clone)]
pub struct Editor {
    pub buffers: Vec<Buffer>,
    pub screen: Screen,
    pub clipboard: String,
    pub current_buffer_index: usize,
}
