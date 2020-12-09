use iced::{
    Text,
    TextInput,
    text_input,
    HorizontalAlignment,
    Length,
    button,
    Button,
    Element,
    Column,
    Row,
};

use crate::message::{Message};
use crate::number_input::NumberInput;

pub struct RandomCreationWidget  {
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
        let size_widget = Row::new()
            .push(Text::new("Grid size: "))
            .push(self.width_input.view(move |width| Message::GridSizeInputChanged{width, height})
            ).push(
                Text::new(" x ")
            ).push(self.height_input.view(move |height| Message::GridSizeInputChanged{width, height}))
            .into();

        let submission = Button::new(
            &mut self.submission_button_state, 
            Text::new("Create Random Puzzle")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill)
            ).width(Length::Fill)
                .on_press(Message::CreateRandomPuzzle{width, height}).into();
        Column::with_children(vec![size_widget, submission]).into()
    }

    pub fn update(&mut self, width: usize, height: usize) {
        self.width_input.update(width);
        self.height_input.update(height);
    }
}
