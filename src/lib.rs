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
    pub line_wrap: bool,
}
impl Editor {
    pub fn with_buffers(&mut self, buffers: Vec<Buffer>) -> &mut Self {
        self.buffers = buffers;
        self
    }
    pub fn with_line_wrap(&mut self, line_wrap: bool) -> &mut Self {
        self.line_wrap = line_wrap;
        self
    }
}

impl Editor {
    pub fn current_buffer(&self) -> &Buffer {
        self.buffers.get(self.current_buffer_index).unwrap()
    }
    pub fn current_buffer_mut(&mut self) -> &mut Buffer {
        self.buffers.get_mut(self.current_buffer_index).unwrap()
    }
    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }
    pub fn screen(&self) -> &Screen {
        &self.screen
    }
    pub async fn save(&self) -> anyhow::Result<()> {
        self.current_buffer().save().await
    }
}
