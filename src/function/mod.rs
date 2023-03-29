use crate::{
    error::{FreightError, OrReturn},
    execution_engine::{evaluate, ExecutionEngine},
    expression::Expression,
    TypeSystem,
};
use std::fmt::Debug;

mod arg_count;
mod function_ref;
mod function_type;
mod function_writer;

pub use arg_count::*;
pub use function_ref::*;
pub use function_type::*;
pub use function_writer::*;

#[derive(Debug)]
pub struct Function<TS: TypeSystem> {
    pub(crate) expressions: Vec<Expression<TS>>,
    pub(crate) variable_count: usize,
    pub(crate) arg_count: ArgCount,
    pub(crate) return_target: usize,
}

impl<TS: TypeSystem> Function<TS> {
    pub fn call(
        &self,
        engine: &mut ExecutionEngine<TS>,
        args: &mut [TS::Value],
        captured: &[TS::Value],
    ) -> Result<TS::Value, FreightError> {
        if !self
            .arg_count
            .valid_arg_count(args.len() - self.variable_count)
        {
            return Err(FreightError::IncorrectArgumentCount {
                expected_min: self.arg_count.min(),
                expected_max: self.arg_count.max(),
                actual: args.len(),
            });
        }
        if self.expressions.is_empty() {
            return Ok(Default::default());
        }

        

        for expr in self.expressions.iter().take(self.expressions.len() - 1) {
            if let Err(FreightError::Return { target }) = evaluate(expr, engine, args, captured) {
                if target == self.return_target {
                    return Ok(std::mem::take(&mut engine.return_value));
                } else {
                    return Err(FreightError::Return { target });
                }
            }
        }
        evaluate(self.expressions.last().unwrap(), engine, args, captured)
            .or_return(self.return_target, engine)
    }
}
