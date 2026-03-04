use chrono::Local;
use ratatui::prelude::*;

use crate::ui::theme::Theme;

pub struct StatusBarWidget<'a> {
    pub nick: &'a str,
    pub channel: Option<&'a str>,
    pub user_count: usize,
    pub lag_ms: Option<u64>,
    pub is_away: bool,
    pub server: &'a str,
    pub modes: &'a str,
    pub channels: &'a [(String, usize)],
}

impl<'a> Widget for StatusBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }

        let style = Theme::status_bar();

        // Fill background
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf[(x, y)].set_style(style);
            }
        }

        // Line 1: status info
        let time = Local::now().format("%H:%M").to_string();
        let mut parts = Vec::new();
        parts.push(format!("[{}]", time));

        let nick_modes = if self.modes.is_empty() {
            format!("[{}]", self.nick)
        } else {
            format!("[{}(+{})]", self.nick, self.modes)
        };
        parts.push(nick_modes);

        if let Some(ch) = self.channel {
            parts.push(format!("[{}({})]", ch, self.user_count));
        }

        if let Some(lag) = self.lag_ms {
            parts.push(format!("[Lag: {}ms]", lag));
        }

        parts.push(format!("[{}]", self.server));

        if self.is_away {
            parts.push("[Away]".to_string());
        }

        let line1 = parts.join(" ");
        let line1_spans = Line::from(Span::styled(line1, style));

        if area.height >= 1 {
            buf.set_line(area.x, area.y, &line1_spans, area.width);
        }

        // Line 2: channel tabs
        if area.height >= 2 && !self.channels.is_empty() {
            let mut spans = Vec::new();
            for (i, (name, unread)) in self.channels.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::styled(" ", style));
                }
                let is_active = self.channel == Some(name.as_str());
                let tab_style = if is_active {
                    Theme::channel_active()
                } else if *unread > 0 {
                    Theme::channel_unread()
                } else {
                    Theme::channel_inactive()
                };
                let label = if *unread > 0 {
                    format!("{}({})", name, unread)
                } else {
                    name.clone()
                };
                spans.push(Span::styled(label, tab_style));
            }
            let line2 = Line::from(spans);
            buf.set_line(area.x, area.y + 1, &line2, area.width);
        }
    }
}
