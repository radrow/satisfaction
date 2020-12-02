use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    FileDropped(PathBuf),
    SolvePuzzle,
}