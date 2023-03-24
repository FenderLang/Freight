use crate::{expression::Expression, TypeSystem, execution_engine::{ExecutionEngine, evaluate}, error::{FreightError, OrReturn}};
use std::fmt::Debug;

mod function_ref;
mod function_type;
mod function_writer;

pub use function_ref::FunctionRef;
pub use function_type::FunctionType;
pub use function_writer::FunctionWriter;

#[derive(Debug)]
pub struct Function<TS: TypeSystem> {
    pub(crate) expressions: Vec<Expression<TS>>,
    pub(crate) stack_size: usize,
    pub(crate) arg_count: usize,
    pub(crate) return_target: usize,
}

impl<TS: TypeSystem> Function<TS> {
    pub fn call(
        &self,
        engine: &mut ExecutionEngine<TS>,
        args: &mut [TS::Value],
        captured: &[TS::Value],
    ) -> Result<TS::Value, FreightError> {
        if args.len() != self.stack_size {
            return Err(FreightError::IncorrectArgumentCount {
                expected: self.arg_count,
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
