use crate::buffer::Buffer;
use crate::ui::Screen;

#[derive(Debug, Default, Clone)]
pub struct Editor {
    pub buffers: Vec<Buffer>,
    pub screen: Screen,
    pub clipboard: String,
    pub current_buffer_index: usize,
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

#[derive(Debug, Default, Clone)]
pub struct EditorBuilder {
    pub buffers: Option<Vec<Buffer>>,
}

impl EditorBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn buffers(&mut self, buffers: Vec<Buffer>) -> &mut Self {
        self.buffers = Some(buffers);
        self
    }
    pub fn build(&mut self) -> Editor {
        Editor {
            buffers: self.buffers.take().unwrap(),
            ..Default::default()
        }
    }
}
