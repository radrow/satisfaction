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

pub enum FieldState {
    Playable(Field),
    Solving,
    Solved,
}

enum GameState {
    FieldAvailable{field: Field, state: FieldState},
    Loading,
    Creating,
    Empty,
}

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

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let field_widget = FieldWidget::new(15, 2, 2);
        let control_widget = ControlWidget::new(180);

        let game = Game {
            state: GameState::Empty,
            log: Log::new(),

            field_widget,
            control_widget,
        };
        (game, Command::none())
    }

    fn title(&self) -> String {
        String::from("Solving Tents")
    }

    ///
    ///
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
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

            Message::SolvePuzzle => {
                let mut cmd = Command::none();
                scoped::scope(|scope| {
                    let (state, hole) = scope.take(&mut self.state);

                    match state {
                        GameState::FieldAvailable {
                            state: FieldState::Playable(old_field),
                            field,
                        } => {
                            hole.fill(
                                GameState::FieldAvailable {
                                    field,
                                    state: FieldState::Solving,
                                }
                            );

                            let mut field = old_field;
                            let fun = lazy(move |_| {
                                if field.solve(&CadicalSolver) {
                                    Message::SolutionFound(field)
                                } else {
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

            Message::GridSizeInputChanged{width, height} => {
                self.control_widget.field_creation_widget.update(width, height)
            },

            Message::CreateRandomPuzzle{width , height} => match self.state {
                GameState::Empty 
                    | GameState::FieldAvailable{state: FieldState::Playable(_), ..}
                    | GameState::FieldAvailable{state: FieldState::Solved, ..} => {
                        self.state = GameState::Creating;
                        let lazy = lazy(move |_| {
                            match create_random_puzzle(height, width) {
                                Ok(field) => Message::FieldLoaded(field),
                                Err(msg) => Message::ErrorOccurred(msg),
                            }
                        });
                        return Command::perform(lazy, identity)
                },
                _ => {},
            },

            Message::ErrorOccurred(error) => {
                self.log.add_error(error);
                self.state = GameState::Empty;
            }
            
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
