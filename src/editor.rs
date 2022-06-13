use crate::buffer::Buffer;
use crate::modes::Mode;
use crate::movement::Movement;
use crate::ui::Screen;
use anyhow::Result;

#[derive(Debug, Default, Clone)]
pub struct Editor {
    pub buffers: Vec<Buffer>,
    pub screen: Screen,
    pub clipboard: String,
    pub current_buffer_index: usize,
    pub mode: Mode,
    pub last_selection: Selection,
}

#[derive(Debug, Default, Clone)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn at_cursor(raw_position: usize) -> Self {
        Self {
            start: raw_position,
            end: raw_position,
        }
    }
    pub fn contains(&self, raw_position: usize) -> bool {
        let min = self.start.min(self.end);
        let max = self.start.max(self.end);
        min <= raw_position && raw_position <= max
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
    pub async fn save(&self) -> Result<()> {
        self.current_buffer().save().await
    }
    pub fn delete_selection(&mut self) -> Result<()> {
        if self.mode != Mode::Visual {
            unreachable!()
        }
        let start = self.last_selection.start;
        let end = self.last_selection.end;
        let min = start.min(end);
        let max = start.max(end);
        Movement::ToRaw(min).do_move(self)?;
        Movement::ToRaw(max).delete(self)?;
        self.mode = Mode::Normal;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct EditorBuilder {
    pub buffers: Option<Vec<Buffer>>,
}

#[derive(Debug)]
struct EmptyBuffers;
impl std::fmt::Display for EmptyBuffers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot build an editor without buffers")
    }
}

impl std::error::Error for EmptyBuffers {}

impl EditorBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn buffers(&mut self, buffers: Vec<Buffer>) -> &mut Self {
        self.buffers = Some(buffers);
        self
    }
    pub fn build(&mut self) -> Result<Editor> {
        Ok(Editor {
            buffers: self.buffers.take().ok_or(EmptyBuffers)?,
            ..Default::default()
        })
    }
}
