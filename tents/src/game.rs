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
    widgets::{
        FieldWidget,
        ControlWidget,
        Log,
    },
};

/// States a field can have
/// w.r.t. the user interaction that is possible.
pub enum FieldState {
    /// Tents can be played, 
    /// i.e. the user can set and unset tents on appropriate places.
    /// The original `Field`, with no tents placed, needs to be preserved
    /// to run a SAT-solver.
    Playable(Field),
    /// The gui is not interactive anymore and the solver is running.
    Solving,
    /// The gui is not interactive but shows a solved field.
    Solved,
}

/// An Enum that categorises all possible states a program can be.
enum GameState {
    /// `FieldAvailable`: A field is avaiable can be played and solved.
    /// 
    /// # Arguments
    /// * `field` - Current field that is shown to the user and contains it changes.
    /// * `state` - State of the current field determining how the user can interacti with it.
    FieldAvailable{field: Field, state: FieldState},
    /// A field is currently loaded from file.
    Loading,
    /// A random field is currently created.
    Creating,
    /// Neither is a field avaiable nor is one loaded.
    Empty,
}

/// Entry point of the whole Tents-application
/// Speaking in Elm's parlance, it is the model of the program.
///
pub struct Game {
    /// Current state of the game determining how it should react on messages.
    state: GameState,
    /// Text messages, i.e. errors or hints, can be push to this log and are shown to the user.
    log: Log,

    /// Graphical representation of the current field.
    field_widget: FieldWidget,
    /// Widget gathering any user interaction that is not directly related to the field,
    /// e.g. a button to solve or create a Tents puzzle.
    control_widget: ControlWidget,
}

impl Game {
    fn is_solvable(&self) -> bool {
        match self.state {
            GameState::FieldAvailable{state: FieldState::Playable(_), ..}   => true,
            _                                                               => false,
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
                            let fun = lazy(move |_| {
                                if let Some(new_field) = field_to_cnf(old_field, &CadicalSolver) {
                                    Message::SolutionFound(new_field)
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
                            match create_random_puzzle(width, height) {
                                Ok(field) => Message::FieldLoaded(field),
                                Err(msg) => Message::ErrorOccurred(msg.to_string()),
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
                self.state = GameState::Empty;
            }
            
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
