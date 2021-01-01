use crate::field::Field;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Message {
    FileDropped(std::path::PathBuf),
    FieldLoaded(Field),

    CreateRandomPuzzle{width: usize, height: usize},
    SolvePuzzle,

    GridSizeInputChanged{width: usize, height: usize},

    ErrorOccurred(String),
}

impl<E: std::error::Error> From<E> for Message {
    fn from(error: E) -> Message {
        Message::ErrorOccurred(error.to_string())
    }
}
