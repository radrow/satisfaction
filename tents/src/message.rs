use crate::field::Field;
use std::path::PathBuf;

/// Fundamental way of communicating for a `iced` application.
#[derive(Debug, Clone)]
pub enum Message {
    /// The user has dropped a file under the path [`PathBuf`].
    /// The game changes to state [`Loading`](crate::game::GameState::Loading),
    /// the file is parsed and a [`Field`] is created.
    FileDropped(PathBuf),
    /// A field that was ordered to be loaded from a file,
    /// is now available as [`Field`].
    /// The game's state changes to
    /// [`FieldAvailable`](crate::game::GameState::FieldAvailable).
    FieldLoaded { field: Field, task_id: usize },

    /// The user has ordered the program to create a random [`Field`]
    /// of specified size.
    /// The game's changes to state [`Creating`](crate::game::GameState::Creating).
    CreateRandomPuzzle { width: usize, height: usize },
    /// The user has ordered the program to solve the current puzzle.
    /// A SAT solver is started asynchronously.
    /// The field's state changes to [`Solving`](crate::game::FieldState::Solving).
    SolvePuzzle,
    /// If the SAT-solver has finished its computation,
    /// it sends this message to inform the program and submit a solve [`Field`].
    /// The field's state changes to [`Solved`](crate::game::FieldState::Solved).
    SolutionFound { field: Field, task_id: usize },

    /// If the user changes the size setting for the random puzzle creation,
    /// this message orders the program to change the respective widget.
    GridSizeInputChanged { width: usize, height: usize },

    /// A field button was pressed put or remove a tent at specified position
    FieldButtonPressed(usize, usize),

    /// If the user changes solver,
    /// this message orders the program to change the respective widget.
    ChangedSolver { new_solver: String },

    /// If an error occurres during the program execution,
    /// e.g. a file wasn't found, a puzzle is unsolvable,
    /// the user is notified via this message.
    ErrorOccurred(String),

    /// A sink message that is sent
    /// if an execution was aborted and its output should be ignored.
    AbortedExecution,
}

impl<E: std::error::Error> From<E> for Message {
    fn from(error: E) -> Message {
        Message::ErrorOccurred(error.to_string())
    }
}
