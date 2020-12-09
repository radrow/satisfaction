use crate::{message::*, puzzle_creation};
use crate::random_creation_widget::RandomCreationWidget;
use iced::button::State;
use iced::{Length, HorizontalAlignment, VerticalAlignment, Element, Button, Text, Column};

pub struct ControlWidget {
    width: Length,
    height: Length,
    text_size: u16,

    pub field_creation_widget: RandomCreationWidget,

    create_custom_field_button: State,
    create_random_field_button: State,
    solve_puzzle_button: State,

    log: Option<String>,
}

impl ControlWidget {
    pub fn new(width: u16, height: u16, text_size: u16) -> ControlWidget {
        ControlWidget {
            width: Length::Units(width),
            height: Length::Units(height),
            field_creation_widget: RandomCreationWidget::new(10, 10),

            create_custom_field_button: State::new(),
            create_random_field_button: State::new(),
            solve_puzzle_button: State::new(),
            text_size,
            log: Some("Drag and drop a tent file!".to_string())
        }
    }

    pub fn view(&mut self, solvable: bool) -> Element<Message> {
        let mut control = Column::new()
            .width(self.width)
            .push(
                self.field_creation_widget.view()
            );
        if solvable {
            control = control.push(
                ControlWidget::button(
                    &mut self.solve_puzzle_button,
                    "Solve Puzzle",
                    Message::SolvePuzzle
                ));
        }
        if let Some(message) = &self.log {
            control = control.push(
                Text::new(message)
                    .size(self.text_size)
            );
        };
        control.into()
    }
    
    fn button<'a>(state: &'a mut State, text: &str, message: Message) -> Button<'a, Message> {
        Button::new(state, 
            Text::new(text)
            .horizontal_alignment(HorizontalAlignment::Center)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .on_press(message)
    }

    pub fn add_to_log<S: Into<String>>(&mut self, message: S) {
        self.log.replace(message.into());
    }
}