use std::fmt;

#[derive(Debug, Clone)]
pub struct KodeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub category: &'static str,
}

impl KodeError {
    pub fn new(
        category: &'static str,
        message: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            category,
            message: message.into(),
            line,
            column,
        }
    }
}

impl fmt::Display for KodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} error at {}:{}: {}",
            self.category,
            self.line,
            self.column,
            self.message
        )
    }
}

impl std::error::Error for KodeError {}