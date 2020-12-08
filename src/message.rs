use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    FileDropped(PathBuf),
    
    CreateCustomPuzzle(usize, usize),
    CreateRandomPuzzle{width: usize, height: usize, num_tent: usize},
    SolvePuzzle,

    GridSizeInputChanged(GridSizeEvent),
}

#[derive(Debug, Clone)]
pub enum GridSizeEvent {
    Submitted,
    WidthChanged(usize),
    HeightChanged(usize),
    TentNumChanged(usize),
}

impl Into<Message> for GridSizeEvent {
    fn into(self) -> Message {
        Message::GridSizeInputChanged(self)
    }
}