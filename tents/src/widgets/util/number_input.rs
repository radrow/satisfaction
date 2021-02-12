use iced::{
    TextInput,
    text_input::State,
    Element
};
use crate::message::Message;

/// A wrapper to the number input,
/// to prohibit non-number input.
pub struct NumberInput {
    state: State,
    pub value: usize,
    default_value: usize,
}

impl NumberInput {
    pub fn new(default_value: usize) -> NumberInput {
        NumberInput {
            state: State::new(),
            value: default_value,
            default_value,
        }
    }
    
    pub fn view<F>(&mut self, on_change: F) -> Element<Message>
    where F: Fn(usize) -> Message + 'static {
        let default_value = self.default_value;
        TextInput::new(
            &mut self.state,
            "",
            self.value.to_string().as_str(),
            move |input| {
                let new_value = input.chars()
                    .filter(|c| c.is_digit(10))
                    .collect::<String>()
                    .parse::<usize>()
                    // If input is invalid, reset text box to default value.
                    .unwrap_or(default_value);
                on_change(new_value)
        }).into()
    }

    pub fn update(&mut self, new_value: usize) {
        self.value = new_value;
    }
}
