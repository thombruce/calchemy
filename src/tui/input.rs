use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

pub struct InputDialog<'a> {
    content: &'a str,
}

impl<'a> InputDialog<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }
}

impl<'a> Widget for InputDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Add Event ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        block.clone().render(area, buf);

        let inner = block.inner(area);

        if !self.content.is_empty() {
            buf.set_string(inner.x, inner.y, self.content, Style::default());
        }
    }
}
