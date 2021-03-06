use crate::{
    game::{Config, FieldState, GameState},
    message::*,
    widgets::RandomCreationWidget,
};

use iced::{
    button, pick_list, Button, Column, Element, HorizontalAlignment, Length, PickList, Text,
};

/// Widget that gathers user input possibilities that are not directly related to field
/// manipulation, e.g. order start a solver and create a new field.
pub struct ControlWidget {
    /// A list containing the names of choosable solvers.
    solver_names: Vec<String>,

    pub selected_solver: String,

    pub field_creation_widget: RandomCreationWidget,

    solve_puzzle_button: button::State,

    solver_choice_list: pick_list::State<String>,

    spacing: u16,
}

impl ControlWidget {
    pub fn new(config: &Config, solver_names: Vec<String>) -> ControlWidget {
        ControlWidget {
            selected_solver: solver_names
                .first()
                .expect("No solver was found!")
                .to_string(),
            solver_names,

            field_creation_widget: RandomCreationWidget::new(5, 5),

            solve_puzzle_button: button::State::default(),
            solver_choice_list: pick_list::State::default(),

            spacing: config.spacing,
        }
    }

    ///  All user input possibilities are ordered vertically:
    ///  * Solver list
    ///  * Specification of random puzzle size
    ///  * Button to create a random puzzle
    ///  * If a field is available: Button to solve a puzzle.
    pub fn view(&mut self, state: &GameState) -> Element<Message> {
        let mut control = Column::new()
            .spacing(self.spacing)
            .push(
                PickList::new(
                    &mut self.solver_choice_list,
                    &self.solver_names,
                    Some(self.selected_solver.clone()),
                    |new_solver| Message::ChangedSolver { new_solver },
                )
                .width(Length::Fill),
            )
            .push(self.field_creation_widget.view());

        match state {
            GameState::FieldAvailable {
                state: FieldState::Playable(_, _),
                ..
            } => {
                control = control.push(ControlWidget::button(
                    &mut self.solve_puzzle_button,
                    "Solve Puzzle",
                    Message::SolvePuzzle,
                ));
            }
            _ => {}
        }
        control.into()
    }

    /// Creates the button to solve a puzzle
    fn button<'a>(
        state: &'a mut button::State,
        text: &str,
        message: Message,
    ) -> Button<'a, Message> {
        Button::new(
            state,
            Text::new(text)
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(message)
    }
}
