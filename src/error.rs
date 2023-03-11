use std::{error::Error, fmt::Display};

use crate::{execution_engine::ExecutionEngine, TypeSystem};

#[derive(Debug, Clone)]
pub enum FreightError {
    InvalidInvocationTarget,
    IncorrectArgumentCount { expected: usize, actual: usize },
    Return { target: usize },
}

impl Display for FreightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInvocationTarget => f.write_str("Cannot invoke non-function values"),
            Self::IncorrectArgumentCount { expected, actual } => {
                write!(f, "Expected {expected} arguments, got {actual}")
            }
            Self::Return { target } => {
                write!(f, "Could not return to target {target}")
            }
        }
    }
}

impl Error for FreightError {}

pub trait OrReturn<TS: TypeSystem> {
    fn or_return(
        self,
        id: usize,
        engine: &mut ExecutionEngine<TS>,
    ) -> Result<TS::Value, FreightError>;
}

impl<TS: TypeSystem> OrReturn<TS> for Result<TS::Value, FreightError> {
    fn or_return(
        self,
        id: usize,
        engine: &mut ExecutionEngine<TS>,
    ) -> Result<<TS as TypeSystem>::Value, FreightError> {
        match self {
            Err(FreightError::Return { target }) if target == id => {
                Ok(std::mem::take(&mut engine.return_value))
            },
            _ => self,
        }
    }
}
