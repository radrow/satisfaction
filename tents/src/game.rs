extern crate iced;

use futures::future::lazy;
use solver::CadicalSolver;
use std::convert::identity;
use iced::{Length, Align};
use iced::{Element, Row, Application, Text, Command, Subscription, HorizontalAlignment, VerticalAlignment, Container};
use iced_native::window::Event;
use take_mut::scoped;

use crate::{
    message::Message, 
    field::*, 
    log::Log,
    control_widget::*, 
    field_widget::FieldWidget,
    puzzle_creation,
};

/// Enum that represents all states that can appear
/// if a field is available.
///
/// * `Playable(field)` - Tents can be played. To solve it after the user has changed the field the original have to be conserved.
/// * `Solving` - The gui is not interactive anymore and the the solver is running.
/// * `Solved` - The gui is not interactive but shows a solved field.
pub enum FieldState {
    Playable(Field),
    Solving,
    Solved,
}

/// An Enum that categorises all possible states a program can be.
///
/// * `FieldAvailable`: A field is avaiable can be played and solved.
/// * `Loading`: A field is currently loaded from file.
/// * `Creating`: A random field is currently created.
/// * `Empty`: Neither is a field avaiable nor is one loaded.
enum GameState {
    FieldAvailable{field: Field, state: FieldState},
    Loading,
    Creating,
    Empty,
}

/// Entry point of the whole Tents-application
/// Speaking in Elm's parlance, it is the model of the program.
///
pub struct Game {
    state: GameState,
    log: Log,

    field_widget: FieldWidget,
    control_widget: ControlWidget,
}

impl Game {
    fn is_solvable(&self) -> bool {
        match self.state {
            GameState::FieldAvailable{state: FieldState::Playable(_), ..} => true,
            _                       => false,
        }
    }
}

impl Application for Game {
    type Executor = iced_futures::executor::Tokio;
    type Message = Message;
    type Flags = ();

    /// Startup of the application.
    /// Here any configuration takes place.
    ///
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let field_widget = FieldWidget::new(15, 2, 2);
        let control_widget = ControlWidget::new(180);

        let game = Game {
            state: GameState::Empty,

            // Log for error messages
            log: Log::new(),

            // View for the field
            field_widget,
            // View for user interaction
            control_widget,
        };
        (game, Command::none())
    }

    fn title(&self) -> String {
        String::from("Solving Tents")
    }

    /// Every time the user interacts with the gui, a system event appears or an asynchronous task
    /// finishes the current model (i.e. `Game`) is updated according to the message those sent.
    ///
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            // If a file is dropped, an asynchronous procedure is called loading this
            // file and converting it into `Field`.
            Message::FileDropped(path) => match self.state {
                GameState::Empty 
                    | GameState::FieldAvailable{state: FieldState::Playable(_), ..}
                    | GameState::FieldAvailable{state: FieldState::Solved, ..} => {

                        self.state = GameState::Loading;
                        return Command::perform(
                            Field::from_file(path),
                            |result| {
                                result.map(Message::FieldLoaded)
                                    .unwrap_or_else(|error| Message::ErrorOccurred(error.to_string()))
                            })
                    },
                _ => {},

            },

            // If the field is finally loaded, the game state is updated now containing the new
            // field.
            Message::FieldLoaded(field) => match self.state {
                GameState::Loading
                    | GameState::Creating => {
                        let old_field = field.clone();
                        self.state = GameState::FieldAvailable {
                            state: FieldState::Playable(old_field),
                            field,
                        };
                    },
                _ => unreachable!(),
            },

            // If the "Solve Puzzle"-button is pressed, this message is sent and the solver is
            // started.
            Message::SolvePuzzle => {
                let mut cmd = Command::none();

                // This is enables swapping member variables for &mut.
                scoped::scope(|scope| {
                    // Take current state leaving `Game` in an inconsistent state
                    let (state, hole) = scope.take(&mut self.state);

                    match state {
                        GameState::FieldAvailable {
                            state: FieldState::Playable(old_field),
                            field,
                        } => {
                            // Restore consistency by replacing former state
                            hole.fill(
                                GameState::FieldAvailable {
                                    field,
                                    state: FieldState::Solving,
                                }
                            );

                            // Start solver with current field
                            let mut field = old_field;
                            let fun = lazy(move |_| {
                                if field.solve(&CadicalSolver) {
                                    // If solving was successfull, send an appropriate message
                                    Message::SolutionFound(field)
                                } else {
                                    // If solving failed, send and error message
                                    Message::ErrorOccurred("No solution for the current Tents puzzle was found!".to_string())
                                }
                            });

                            cmd = Command::perform(fun, identity);
                        },
                        _ => unreachable!(),
                    };
                });
                return cmd;
            },

            // If the user has input a different size, the view have to be updated.
            Message::GridSizeInputChanged{width, height} => {
                self.control_widget.field_creation_widget.update(width, height)
            },

            // If the user orders a random puzzle, 
            // start creation as an asynchronous task
            // and inform the user that one is creating
            Message::CreateRandomPuzzle{width , height} => match self.state {
                GameState::Empty 
                    | GameState::FieldAvailable{state: FieldState::Playable(_), ..}
                    | GameState::FieldAvailable{state: FieldState::Solved, ..} => {

                        self.state = GameState::Creating;

                        let lazy = lazy(move |_| {
                            match puzzle_creation::create_random_puzzle(height, width) {
                                Ok(field) => Message::FieldLoaded(field),
                                Err(msg) => Message::ErrorOccurred(msg),
                            }
                        });
                        return Command::perform(lazy, identity)
                },
                _ => {},
            },

            // If an error occurres log it to the screen.
            // TODO: Reset state appropriately
            Message::ErrorOccurred(error) => {
                self.log.add_error(error);
            },
            
            // If a solution was found,
            // replace current field with the new, solved one.
            Message::SolutionFound(field) => {
                self.state = GameState::FieldAvailable {
                    field, 
                    state: FieldState::Solved,
                }
            }
        };
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Row::new()
        .align_items(Align::Start)
        .push(self.control_widget.view(self.is_solvable(), &self.log))
        .push(Container::new(
            match &mut self.state {
                GameState::Empty => Element::from(
                    Text::new("Drag and drop a file!")
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center)
                ),
                GameState::Loading => Element::from(
                    Text::new("Loading puzzle ...")
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center)
                ),
                GameState::Creating => Element::from(
                    Text::new("Creating random puzzle ...")
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center)
                ),
                GameState::FieldAvailable{field, ..} => self.field_widget.view(&field),
            }).center_x()
                .center_y()
                .width(Length::Fill)
                .height(Length::Fill)
        ).padding(10)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events_with(
            |event, _| {
                match event {
                    iced_native::Event::Window(Event::FileDropped(path)) => Some(Message::FileDropped(path)),
                    _ => None
                }
            })
    }
}
