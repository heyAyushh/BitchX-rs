use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::irc::channel::ChannelUser;
use crate::ui::theme::Theme;

pub struct NickListWidget<'a> {
    pub users: &'a [&'a ChannelUser],
    pub title: &'a str,
}

impl<'a> NickListWidget<'a> {
    pub fn new(users: &'a [&'a ChannelUser], title: &'a str) -> Self {
        Self { users, title }
    }

    fn user_style(user: &ChannelUser) -> Style {
        match user.prefix {
            Some('~' | '&' | '@' | '%') => Theme::nick_op(),
            Some('+') => Theme::nick_voice(),
            _ => Theme::nick_normal(),
        }
    }

    fn format_user(user: &ChannelUser) -> String {
        match user.prefix {
            Some(p) => format!("{}{}", p, user.nick),
            None => format!(" {}", user.nick),
        }
    }
}

impl<'a> Widget for NickListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title)
            .title_style(Theme::nick_list_header())
            .borders(Borders::ALL)
            .border_style(Theme::border());

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let visible = inner.height as usize;
        let items: Vec<ListItem<'_>> = self
            .users
            .iter()
            .take(visible)
            .map(|u| {
                let style = Self::user_style(u);
                let text = Self::format_user(u);
                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items);
        Widget::render(list, inner, buf);
    }
}
