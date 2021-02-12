#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum LogType {
    Error,
    Hint,
}

pub struct Log(pub Vec<(LogType, String)>);

impl Log {
    pub fn new() -> Log {
        Log(Vec::new())
    }

    fn add_message(&mut self, msg: String, type_: LogType) {
        self.0.push((type_, msg));
    }

    #[allow(dead_code)]
    pub fn add_hint(&mut self, msg: String) {
        self.add_message(msg, LogType::Hint);
    }

    pub fn add_error(&mut self, msg: String) {
        self.add_message(msg, LogType::Error);
    }
}
