use iced::{
    widget::{Column, Text},
    Element, Color
};
use crate::message::Message;

#[derive(Debug, Clone, Copy)]
enum LogType {
    Error,
    Hint,
}

struct Log(LogType, String);

pub struct LogWidget {
    log: Vec<Log>,
    font_size: u16,
}

impl LogWidget {
    pub fn new() -> LogWidget {
        LogWidget {
            log: Vec::new(),
            font_size: 12, // TODO: Make font size changeable
        }
    }

    fn add_message(&mut self, msg: String, type_: LogType) {
        self.log.push(Log(type_, msg));
    }

    pub fn add_hint(&mut self, msg: String) {
        self.add_message(msg, LogType::Hint);
    }

    pub fn add_error(&mut self, msg: String) {
        self.add_message(msg, LogType::Error);
    }

    pub fn view(&mut self) -> Element<Message> {
        Column::with_children(
            self.log.iter()
                .map(|Log(type_, msg)|{
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
