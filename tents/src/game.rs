extern crate iced;

use futures::{
    future::{abortable, lazy, AbortHandle, Abortable},
    Future,
};
use solver::solvers::InterruptibleSolver;
use std::{collections::HashMap, sync::Arc};

use iced::{Align, Column, Length};
use iced::{Application, Command, Container, Element, Row, Subscription, Text};
use iced_native::window::Event;
use take_mut::scoped;
use tokio::sync::RwLock;

use crate::{
    field::*,
    message::Message,
    widgets::{ControlWidget, FieldWidget, Log, LogWidget},
};

/// A list of named solvers that can be used to solve the Tents puzzle.
pub type SolverList = HashMap<&'static str, Box<dyn InterruptibleSolver>>;

/// Gathers various values that define how the program looks.
pub struct Config {
    pub cell_size: Length,
    pub cell_spacing: u16,
    pub count_font_size: u16,
    pub solvers: SolverList,
    pub log_field_ratio: (u16, u16),
    pub control_field_ratio: (u16, u16),
    pub spacing: u16,
    pub padding: u16,
    pub button_font_size: u16,
    pub log_font_size: u16,
    pub scrollbar_width: u16,
    pub scrollbar_margin: u16,
}

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
pub enum GameState {
    /// `FieldAvailable`: A field is avaiable can be played and solved.
    ///
    /// # Arguments
    /// * `field` - Current field that is shown to the user and contains it changes.
    /// * `state` - State of the current field determining how the user can interacti with it.
    FieldAvailable { field: Field, state: FieldState },
    /// A field is currently loaded from file.
    Loading,
    /// A random field is currently created.
    Creating,
    /// Neither is a field avaiable nor is one loaded.
    Empty,
}

/// A `CancelHandle` is necessary to make long lasting computations
/// like solving or tent creation abortable.
/// At first the used futures have to be abortable,
/// there must also be a possibility to ignore results of asynchronous computations
/// that are not valid anymore.
/// For example, if one starts a solver,
/// creates a new tent
/// and starts a new solving process for this,
/// the first one must be ignored.
struct CancelHandle {
    /// A handle to the newest asynchronous computation.
    abort_handle: Option<AbortHandle>,
    /// Currently valid id.
    /// It is necessary to check old computations that
    /// finished before being aborted.
    /// Only the the newest task is accepted.
    current_task_id: usize,
}

impl CancelHandle {
    fn new() -> CancelHandle {
        CancelHandle {
            abort_handle: None,
            current_task_id: 0,
        }
    }

    /// Registers a future to make it abortable
    fn register<F: Future>(&mut self, future: F) -> (Abortable<F>, usize) {
        // Make future abortable
        let (abortable_future, handle) = abortable(future);
        // Abort old futures and save new one
        if let Some(old_handle) = self.abort_handle.take() {
            old_handle.abort();
        }
        self.abort_handle = Some(handle);
        // Invalidate old futures that might have finished and
        // could thus not be aborted.
        self.current_task_id = self.current_task_id.wrapping_add(1);
        (abortable_future, self.current_task_id)
    }

    /// A asynchronous computation with id `task_id` finishes without being
    /// aborted. If the `task_id` is still valid,
    /// it is the newest task and `true` is returned
    /// otherwise `false`.
    fn finish_task(&mut self, task_id: usize) -> bool {
        if self.current_task_id == task_id {
            self.abort_handle = None;
            true
        } else {
            false
        }
    }
}

/// Entry point of the whole Tents-application
/// Speaking in Elm's parlance, it is the model of the program.
pub struct Game {
    solvers: Arc<RwLock<SolverList>>,

    /// Current state of the game determining how it should react on messages.
    state: GameState,
    /// Text messages, i.e. errors or hints, can be push to this log and are shown to the user.
    log: Log,

    /// Graphical representation of the current field.
    field_widget: FieldWidget,

    /// Widget gathering any user interaction that is not directly related to the field,
    /// e.g. a button to solve or create a Tents puzzle.
    control_widget: ControlWidget,

    log_widget: LogWidget,

    log_field_ratio: (u16, u16),
    control_field_ratio: (u16, u16),
    padding: u16,

    cancel_handle: CancelHandle,
}

/// The entry point of a gui for the `iced` framework
/// is the implementation of the [`Application`] trait.
impl Application for Game {
    type Executor = iced_futures::executor::Tokio;
    type Message = Message;
    type Flags = Config;

    /// Startup of the application.
    /// Here any configuration takes place.
    fn new(config: Config) -> (Self, Command<Self::Message>) {
        let field_widget = FieldWidget::new(&config);
        let control_widget = ControlWidget::new(
            &config,
            config
                .solvers
                .keys()
                .map(|name| (*name).to_string())
                .collect::<Vec<_>>(),
        );

        let log_widget = LogWidget::new(&config);

        let game = Game {
            solvers: Arc::new(RwLock::new(config.solvers)),

            state: GameState::Empty,

            log: Log::new(),

            field_widget,
            control_widget,
            log_widget,

            log_field_ratio: config.log_field_ratio,
            control_field_ratio: config.control_field_ratio,
            padding: config.padding,

            cancel_handle: CancelHandle::new(),
        };
        (game, Command::none())
    }

    fn title(&self) -> String {
        String::from("Solving Tents")
    }

    /// Every time the user interacts with the gui, a system event appears or an asynchronous task
    /// finishes the current model (i.e. `Game`) is updated according to the message those sent.
    /// If asynchronous work has to be done, it is wrapped into a [`Command`].
    ///
    /// # Arguments
    ///
    /// * `message` - A message that was sent due to an event and that orders the game to change
    ///               its state accordingly.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            // If a file is dropped, an asynchronous procedure is called loading this
            // file and converting it into `Field`.
            Message::FileDropped(path) => {
                self.state = GameState::Loading;
                let (fut, task_id) = self.cancel_handle.register(Field::from_file(path));
                return Command::perform(fut, move |result| match result {
                    Ok(Ok(new_field)) => Message::FieldLoaded {
                        field: new_field,
                        task_id,
                    },
                    Ok(Err(msg)) => Message::ErrorOccurred(msg.to_string()),
                    Err(_) => Message::AbortedExecution,
                });
            }

            // If the field is finally loaded, the game state is updated now containing the new
            // field.
            Message::FieldLoaded { field, task_id } if self.cancel_handle.finish_task(task_id) => {
                match self.state {
                    GameState::Loading | GameState::Creating => {
                        let old_field = field.clone();
                        self.state = GameState::FieldAvailable {
                            state: FieldState::Playable(old_field),
                            field,
                        };
                    }
                    _ => unreachable!(),
                }
            }

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
                            hole.fill(GameState::FieldAvailable {
                                field,
                                state: FieldState::Solving,
                            });

                            // Get selected solver
                            let solvers = self.solvers.clone();
                            let solver_name = self.control_widget.selected_solver.clone();

                            // Register abortable future
                            let (fut, task_id) = self.cancel_handle.register(async move {
                                let solvers = solvers.read().await;

                                let solver = solvers
                                    .get(solver_name.as_str())
                                    .expect("Specified solver was not found!");

                                field_to_cnf(old_field, &solver).await
                            });

                            // Send command to perform solving in background.
                            cmd = Command::perform(fut, move |result| match result {
                                Ok(Some(new_field)) => Message::SolutionFound {
                                    field: new_field,
                                    task_id,
                                },
                                Ok(None) => Message::ErrorOccurred(
                                    "No solution for the current Tents puzzle was found!"
                                        .to_string(),
                                ),
                                Err(_) => Message::AbortedExecution,
                            });
                        }
                        _ => unreachable!(),
                    };
                });
                return cmd;
            }

            // If the user has input a different size, the view have to be updated.
            Message::GridSizeInputChanged { width, height } => self
                .control_widget
                .field_creation_widget
                .update(width, height),

            // If the user orders a random puzzle,
            // start creation as an asynchronous task
            // and inform the user that one is creating
            Message::CreateRandomPuzzle { width, height } => {
                self.state = GameState::Creating;
                let (fut, task_id) = self
                    .cancel_handle
                    .register(lazy(move |_| create_random_puzzle(width, height)));

                return Command::perform(fut, move |result| match result {
                    Ok(Ok(new_field)) => Message::FieldLoaded {
                        field: new_field,
                        task_id,
                    },
                    Ok(Err(msg)) => Message::ErrorOccurred(msg.to_string()),
                    Err(_) => Message::AbortedExecution,
                });
            }

            // If an error occurres log it to the screen.
            Message::ErrorOccurred(error) => {
                self.log.add_error(error);
                self.state = GameState::Empty;
            }

            // If a solution was found,
            // replace current field with the new, solved one.
            Message::SolutionFound { field, task_id }
                if self.cancel_handle.finish_task(task_id) =>
            {
                self.state = GameState::FieldAvailable {
                    field,
                    state: FieldState::Solved,
                }
            }

            Message::ChangedSolver { new_solver } => {
                self.control_widget.selected_solver = new_solver;
            }

            _ => {}
        };
        Command::none()
    }

    /// The graphical representation of the current model.
    fn view(&mut self) -> Element<Self::Message> {
        Column::new()
            .push(
                Row::new()
                    .align_items(Align::Start)
                    .push(
                        Container::new(self.control_widget.view(&self.state))
                            .width(Length::FillPortion(self.control_field_ratio.0)),
                    )
                    .push(
                        Container::new(match &mut self.state {
                            GameState::Empty => Element::from(Text::new("Drag and drop a file!")),
                            GameState::Loading => Element::from(Text::new("Loading puzzle ...")),
                            GameState::Creating => {
                                Element::from(Text::new("Creating random puzzle ..."))
                            }
                            GameState::FieldAvailable { field, .. } => {
                                self.field_widget.view(&field)
                            }
                        })
                        .center_x()
                        .center_y()
                        .width(Length::FillPortion(self.control_field_ratio.1))
                        .height(Length::Fill),
                    )
                    .padding(self.padding)
                    .height(Length::FillPortion(self.log_field_ratio.1)),
            )
            .push(
                Container::new(self.log_widget.view(&self.log))
                    .height(Length::FillPortion(self.log_field_ratio.0))
                    .width(Length::Fill)
                    .align_y(Align::End),
            )
            .height(Length::Fill)
            .padding(self.padding)
            .into()
    }

    /// Non-widget-related events,
    /// e.g. dropping a file,
    /// can be checked by a subscription.
    /// An appropriate message is sent to process it.
    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events_with(|event, _| {
            match event {
                // If a file is dropped, sent an appropriate message
                iced_native::Event::Window(Event::FileDropped(path)) => {
                    Some(Message::FileDropped(path))
                }
                _ => None,
            }
        })
    }
}
