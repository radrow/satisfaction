use iced::{
    widget::{Column, Text},
    Element, Color
};
use crate::message::Message;
use super::{Log, LogType};

pub struct LogWidget {
    font_size: u16,
}

impl LogWidget {
    pub fn new(font_size: u16) -> LogWidget {
        LogWidget {
            font_size,
        }
    }

    pub fn view(&mut self, log: &Log) -> Element<Message> {
        Column::with_children(
            log.0.iter()
                .map(|(type_, msg)|{
                    let (tag, color) = match type_ {
                        LogType::Hint => ("Hint", Color::BLACK),
                        LogType::Error => ("Error", Color::from_rgb(1., 0., 0.)),
                    };
                    Text::new(format!("[{}] {}", tag, msg))
                        .color(color)
                        .size(self.font_size)
                        .into()
                }).collect()
        ).into()
    }
}
