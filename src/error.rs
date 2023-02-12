use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub enum FreightError {
    InvalidInvocationTarget,
    IncorrectArgumentCount {expected: usize, actual: usize},
}

impl Display for FreightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for FreightError {}