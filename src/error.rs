use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub enum FreightError {
    InvalidInvocationTarget,
    IncorrectArgumentCount { expected: usize, actual: usize },
}

impl Display for FreightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInvocationTarget => f.write_str("Cannot invoke non-function values"),
            Self::IncorrectArgumentCount { expected, actual } => {
                write!(f, "Expected {expected} arguments, got {actual}")
            }
        }
    }
}

impl Error for FreightError {}
