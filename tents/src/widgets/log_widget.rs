use iced::{
    Text, Element, Color, Scrollable,
    scrollable::State,
    Length,
};
use crate::message::Message;
use crate::game::Config;
use super::{Log, LogType};

pub struct LogWidget {
    font_size: u16,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scrollbar_state: State,
}

impl LogWidget {
    pub fn new(config: &Config) -> LogWidget {
        LogWidget {
            font_size: config.log_font_size,
            scrollbar_width: config.scrollbar_width,
            scrollbar_margin: config.scrollbar_margin,
            scrollbar_state: State::new(),
        }
    }

    pub fn view(&mut self, log: &Log) -> Element<Message> {
        let font_size = self.font_size;
        let scrollbar = Scrollable::new(&mut self.scrollbar_state)
            .scroller_width(self.scrollbar_width)
            .scrollbar_margin(self.scrollbar_margin)
            .width(Length::Fill)
            .height(Length::Fill);

        log.0.iter()
            .rev()
            .fold(scrollbar, |scrollbar, (type_, msg)| {
                let (tag, color) = match type_ {
                    LogType::Hint => ("Hint", Color::BLACK),
                    LogType::Error => ("Error", Color::from_rgb(1., 0., 0.)),
                };
                let text = Text::new(format!("[{}] {}", tag, msg))
                    .color(color)
                    .size(font_size);
                scrollbar.push(text)
            }).into()
    }
}
