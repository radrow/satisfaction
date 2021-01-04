use crate::{message::*, log_widget::LogWidget};
use crate::random_creation_widget::RandomCreationWidget;
use iced::button::State;
use iced::{Length, HorizontalAlignment, Element, Button, Text, Column};


pub struct ControlWidget {
    width: Length,

    pub field_creation_widget: RandomCreationWidget,
    pub log_widget: LogWidget,

    solve_puzzle_button: State,
}

impl ControlWidget {
    pub fn new(width: u16) -> ControlWidget {
        ControlWidget {
            width: Length::Units(width),
            field_creation_widget: RandomCreationWidget::new(10, 10),
            log_widget: LogWidget::new(),

            solve_puzzle_button: State::new(),
        }
    }

    pub fn view(&mut self, solvable: bool) -> Element<Message> {
        let mut control = Column::new()
            .width(self.width)
            .push(self.field_creation_widget.view())
            .push(self.log_widget.view());

        if solvable {
            control = control.push(
                ControlWidget::button(
                    &mut self.solve_puzzle_button,
                    "Solve Puzzle",
                    Message::SolvePuzzle
                ));
        }
        control.spacing(10).into()
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
