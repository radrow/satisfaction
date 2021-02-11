use crate::{
    message::*, 
    widgets::{LogWidget, RandomCreationWidget},
    game::{GameState, FieldState},
};
use super::Log;

use iced::{Length, HorizontalAlignment, Element, Button, Text, Column, pick_list, PickList, button};

pub struct ControlWidget {
    solver_names: Vec<String>,
    pub selected_solver: String,

    width: Length,

    pub field_creation_widget: RandomCreationWidget,
    log_widget: LogWidget,

    solve_puzzle_button: button::State,
    solver_choice_list: pick_list::State<String>,
}

impl ControlWidget {
    pub fn new(width: u16, log_font_size: u16, solver_names: Vec<String>) -> ControlWidget {
        ControlWidget {
            selected_solver: solver_names.first()
                .expect("No solver was found!")
                .to_string(),
            solver_names,

            width: Length::Units(width),
            field_creation_widget: RandomCreationWidget::new(10, 10),
            log_widget: LogWidget::new(log_font_size),

            solve_puzzle_button: button::State::default(),
            solver_choice_list: pick_list::State::default(),
        }
    }


    pub fn view(&mut self, state: &GameState, log: &Log) -> Element<Message> {
        let mut control = Column::new()
            .spacing(10)
            .width(self.width)
            .push(PickList::new(
                    &mut self.solver_choice_list,
                    &self.solver_names,
                    Some(self.selected_solver.clone()),
                    |new_solver| Message::ChangedSolver{new_solver},
            ))
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

    
    fn button<'a>(state: &'a mut button::State, text: &str, message: Message) -> Button<'a, Message> {
        Button::new(state, 
            Text::new(text)
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill)
        )
        .width(Length::Fill)
        .on_press(message)
    }
}
