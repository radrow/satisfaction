use iced::{button, Button, Column, Element, HorizontalAlignment, Length, Row, Text};

use super::util::NumberInput;
use crate::message::Message;

/// A random creation widget that enables
/// the user to specify the size of a puzzle
/// to be created randomly.
pub struct RandomCreationWidget {
    width_input: NumberInput,
    height_input: NumberInput,

    submission_button_state: button::State,
}

impl RandomCreationWidget {
    pub fn new(start_width: usize, start_height: usize) -> RandomCreationWidget {
        RandomCreationWidget {
            submission_button_state: button::State::new(),
            width_input: NumberInput::new(start_width),
            height_input: NumberInput::new(start_height),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let width = self.width_input.value;
        let height = self.height_input.value;
        let size_widget: Row<Message> = Row::new()
            .push(Text::new("Grid size: "))
            .push(
                self.width_input
                    .view(move |width| Message::GridSizeInputChanged { width, height }),
            )
            .push(Text::new(" x "))
            .push(
                self.height_input
                    .view(move |height| Message::GridSizeInputChanged { width, height }),
            )
            .into();

        let submission: Button<Message> = Button::new(
            &mut self.submission_button_state,
            Text::new("Create Random Puzzle")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(Message::CreateRandomPuzzle { width, height })
        .into();

        Column::new()
            .push(size_widget)
            .push(submission)
            .spacing(10)
            .into()
    }

    pub fn update(&mut self, width: usize, height: usize) {
        self.width_input.update(width);
        self.height_input.update(height);
    }
}
