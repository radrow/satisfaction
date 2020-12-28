use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    FileDropped(PathBuf),
    
    //CreateCustomPuzzle(usize, usize),
    CreateRandomPuzzle{width: usize, height: usize},
    SolvePuzzle,

    GridSizeInputChanged{width: usize, height: usize},
}