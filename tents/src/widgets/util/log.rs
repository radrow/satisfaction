#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
/// Classication of a message so 
/// the user can directly see if
/// it is a error or just and general information.
pub enum LogType {
    Error,
    Hint,
}

/// Simple stack of text messages to inform 
/// the user about interesting event or errors.
pub struct Log(pub Vec<(LogType, String)>);

impl Log {
    pub fn new() -> Log {
        Log(Vec::new())
    }

    /// Append a message to the log stack
    fn add_message(&mut self, msg: String, type_: LogType) {
        self.0.push((type_, msg));
    }

    #[allow(dead_code)]
    /// Append a general hint to the log stack
    pub fn add_hint(&mut self, msg: String) {
        self.add_message(msg, LogType::Hint);
    }

    /// Append an error message to the log stack
    pub fn add_error(&mut self, msg: String) {
        self.add_message(msg, LogType::Error);
    }
}
