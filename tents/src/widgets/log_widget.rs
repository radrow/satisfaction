use iced::{
    Text, Element, Color, Scrollable,
    scrollable::State,
    Length,
};
use crate::message::Message;
use super::{Log, LogType};

pub struct LogWidget {
    font_size: u16,
    scrollbar_state: State,
}

impl LogWidget {
    pub fn new(font_size: u16) -> LogWidget {
        LogWidget {
            font_size,
            scrollbar_state: State::new(),
        }
    }

    pub fn view(&mut self, log: &Log) -> Element<Message> {
        let font_size = self.font_size;
        let scrollbar = Scrollable::new(&mut self.scrollbar_state)
            .scroller_width(5)
            .spacing(5)
            .scrollbar_margin(2)
            .width(Length::Fill)
            .height(Length::Units(50));
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
