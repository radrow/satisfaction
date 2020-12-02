use crate::message::*;
use iced::button::State;
use iced::{Length, HorizontalAlignment, VerticalAlignment, Element, Button, Text, Column};

pub struct ControlWidget {
    width: Length,
    height: Length,
    solve_button: State,
    text_size: u16,
    pub log: Option<String>,
}

impl ControlWidget {
    pub fn new(width: u16, height: u16, text_size: u16) -> ControlWidget {
        ControlWidget {
            width: Length::Units(width),
            height: Length::Units(height),
            solve_button: State::new(),
            text_size,
            log: Some("Drag and drop a tent file!".to_string())
        }
    }

    pub fn draw(&mut self) -> Element<Message> {
        let mut control = Column::new()
            .width(self.width)
            .push(
                Button::new(&mut self.solve_button, Text::new("Solve Puzzle!"))
                .on_press(Message::SolvePuzzle)
            );
        if let Some(message) = &self.log {
            control = control.push(
                Text::new(message)
                    .size(self.text_size)
            );
        };
        control.into()
    }
}