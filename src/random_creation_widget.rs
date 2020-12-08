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

use crate::message::{Message, GridSizeEvent};

enum Task{

}

pub struct RandomCreationWidget  {
    width_state: text_input::State,
    height_state: text_input::State,
    num_tent_state: text_input::State,

    submission_button_state: button::State,

    default_width: usize,
    default_height: usize,
    default_num_tent: usize,

    width: usize,
    height: usize,
    num_tent: usize,
}

impl RandomCreationWidget {
    pub fn new(start_width: usize, start_height: usize, start_num_tent: usize) -> RandomCreationWidget {
        RandomCreationWidget {
            width_state: text_input::State::new(),
            height_state: text_input::State::new(),
            submission_button_state: button::State::new(),

            num_tent_state: text_input::State::new(),

            default_width: start_width,
            default_height: start_height,
            default_num_tent: start_num_tent,
            width: start_width,
            height: start_height,
            num_tent: start_num_tent,
        }
    }

    pub fn widget(&mut self) -> Element<Message> {
        Column::new()
            .push(self.grid_size_input())
            .into()
    }

    pub fn update(&mut self, event: GridSizeEvent) {
        match event {
            GridSizeEvent::Submitted => {}
            GridSizeEvent::WidthChanged(width) => {self.width = width}
            GridSizeEvent::HeightChanged(height) => {self.height = height}
            GridSizeEvent::TentNumChanged(num_tens) => {self.num_tent = num_tens}
        }
    }
    fn grid_size_input(&mut self) -> Element<Message> {
        let size_widget = Row::new()
            .push(Text::new("Grid size: "))
            .push(RandomCreationWidget::number_input(
                &mut self.width_state, 
                "Width", 
                self.width, 
                self.default_width,
                |width| GridSizeEvent::WidthChanged(width)
            )).push(
                Text::new(" x ")
            ).push( RandomCreationWidget::number_input(
                &mut self.height_state, 
                "Height",
                self.height,
                self.default_height,
                |height| GridSizeEvent::HeightChanged(height)
            )).into();

        let tent_widget = Row::new()
            .push(Text::new("Tents/Trees: "))
            .push(RandomCreationWidget::number_input(
                &mut self.num_tent_state,
                "",
                self.num_tent,
                self.default_num_tent,
                |num_tent| GridSizeEvent::TentNumChanged(num_tent)),
            ).into();

        let submission = Button::new(
            &mut self.submission_button_state, 
            Text::new("Create Random Puzzle")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill)
            ).width(Length::Fill)
                .on_press(Message::CreateRandomPuzzle{
                    width: self.width, 
                    height: self.height, 
                    num_tent: self.num_tent
            }).into();
        Column::with_children(vec![size_widget, tent_widget, submission]).into()
    }

    fn number_input<'a>(state: &'a mut text_input::State, placeholder: &str, value: usize, default_value: usize, on_change: fn(usize) -> GridSizeEvent) -> Element<'a, Message>{
        TextInput::new(
            state,
            placeholder,
            value.to_string().as_str(),
            move |input| {
                let new_value = input.chars()
                    .filter(|c| c.is_digit(10))
                    .collect::<String>()
                    .parse::<usize>()
                    .unwrap_or(default_value);
                on_change(new_value).into()
            }).into()
    }
}
