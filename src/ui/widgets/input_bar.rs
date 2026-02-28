use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::ui::theme::Theme;

pub struct InputBarWidget<'a> {
    pub text: &'a str,
    pub cursor_pos: usize,
    pub prompt: &'a str,
}

impl<'a> InputBarWidget<'a> {
    pub fn new(text: &'a str, cursor_pos: usize) -> Self {
        Self {
            text,
            cursor_pos,
            prompt: "> ",
        }
    }

    pub fn prompt(mut self, prompt: &'a str) -> Self {
        self.prompt = prompt;
        self
    }
}

impl<'a> Widget for InputBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Theme::input_bar();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border());
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let display = format!("{}{}", self.prompt, self.text);
        let line = Line::from(Span::styled(display, style));
        let paragraph = Paragraph::new(line);
        paragraph.render(inner, buf);

        let cursor_x = inner.x + self.prompt.len() as u16 + self.cursor_pos as u16;
        let cursor_y = inner.y;
        if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
            buf[(cursor_x, cursor_y)].set_style(Style::default().add_modifier(Modifier::REVERSED));
        }
    }
}
