use crate::{
    message::*, 
    widgets::{LogWidget, RandomCreationWidget},
    game::{GameState, FieldState},
};
use super::Log;

use iced::button::State;
use iced::{Length, HorizontalAlignment, Element, Button, Text, Column};


pub struct ControlWidget {
    width: Length,

    pub field_creation_widget: RandomCreationWidget,
    log_widget: LogWidget,

    solve_puzzle_button: State,
}

impl ControlWidget {
    pub fn new(width: u16, log_font_size: u16) -> ControlWidget {
        ControlWidget {
            width: Length::Units(width),
            field_creation_widget: RandomCreationWidget::new(10, 10),
            log_widget: LogWidget::new(log_font_size),

            solve_puzzle_button: State::new(),
        }
    }

    pub fn view(&mut self, state: &GameState, log: &Log) -> Element<Message> {
        let mut control = Column::new()
            .spacing(10)
            .width(self.width)
            .push(self.field_creation_widget.view());

        match state {
            GameState::FieldAvailable {
                state: FieldState::Playable(_),
                ..
            } => {
                control = control.push(
                    ControlWidget::button(
                        &mut self.solve_puzzle_button,
                        "Solve Puzzle",
                        Message::SolvePuzzle
                    ));
            },
            _ => {},
        }

        control.push(self.log_widget.view(log))
            .into()
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
}
