use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::irc::channel::{ChatMessage, MessageKind};
use crate::ui::theme::Theme;

pub struct ChatWidget<'a> {
    pub messages: &'a [ChatMessage],
    pub scroll_offset: usize,
    pub show_timestamps: bool,
}

impl<'a> ChatWidget<'a> {
    pub fn new(messages: &'a [ChatMessage]) -> Self {
        Self {
            messages,
            scroll_offset: 0,
            show_timestamps: true,
        }
    }

    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub fn show_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    fn format_message(&self, msg: &ChatMessage) -> Line<'a> {
        let mut spans = Vec::new();

        if self.show_timestamps {
            let ts = msg.timestamp.format("[%H:%M] ").to_string();
            spans.push(Span::styled(ts, Theme::timestamp()));
        }

        let style = Theme::message_style(&msg.kind);
        let nick_style = Style::default().fg(Theme::nick_color(&msg.sender));

        match msg.kind {
            MessageKind::Normal => {
                spans.push(Span::styled(format!("<{}>", msg.sender), nick_style));
                spans.push(Span::styled(format!(" {}", msg.content), style));
            }
            MessageKind::Action => {
                spans.push(Span::styled(
                    format!("* {} {}", msg.sender, msg.content),
                    style,
                ));
            }
            MessageKind::Notice => {
                spans.push(Span::styled(
                    format!("-{}- {}", msg.sender, msg.content),
                    style,
                ));
            }
            MessageKind::Join => {
                spans.push(Span::styled(
                    format!("--> {} has joined {}", msg.sender, msg.content),
                    style,
                ));
            }
            MessageKind::Part => {
                spans.push(Span::styled(
                    format!("<-- {} has left {}", msg.sender, msg.content),
                    style,
                ));
            }
            MessageKind::Quit => {
                spans.push(Span::styled(
                    format!("<-- {} has quit ({})", msg.sender, msg.content),
                    style,
                ));
            }
            MessageKind::Kick => {
                spans.push(Span::styled(
                    format!("<<< {} was kicked: {}", msg.sender, msg.content),
                    style,
                ));
            }
            MessageKind::Mode => {
                spans.push(Span::styled(
                    format!("*** {} sets mode {}", msg.sender, msg.content),
                    style,
                ));
            }
            MessageKind::Topic => {
                spans.push(Span::styled(
                    format!("*** {} changed the topic to: {}", msg.sender, msg.content),
                    style,
                ));
            }
            MessageKind::Nick => {
                spans.push(Span::styled(
                    format!("*** {} is now known as {}", msg.sender, msg.content),
                    style,
                ));
            }
            MessageKind::System => {
                spans.push(Span::styled(format!("*** {}", msg.content), style));
            }
            MessageKind::Error => {
                spans.push(Span::styled(format!("!!! {}", msg.content), style));
            }
        }

        Line::from(spans)
    }
}

impl<'a> Widget for ChatWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border());

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let visible_height = inner.height as usize;
        let total = self.messages.len();

        let end = if self.scroll_offset == 0 {
            total
        } else {
            total.saturating_sub(self.scroll_offset)
        };
        let start = end.saturating_sub(visible_height);

        let lines: Vec<Line<'_>> = self.messages[start..end]
            .iter()
            .map(|m| self.format_message(m))
            .collect();

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}
