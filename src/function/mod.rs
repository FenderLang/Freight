use crate::{
    error::{FreightError, OrReturn},
    execution_engine::{evaluate, ExecutionEngine},
    expression::Expression,
    value::Value,
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
    pub(crate) stack_size: usize,
    pub(crate) arg_count: ArgCount,
    pub(crate) return_target: usize,
}

impl<TS: TypeSystem> Function<TS> {
    pub fn call(
        &self,
        engine: &mut ExecutionEngine<TS>,
        args: &mut [TS::Value],
        captured: &[TS::Value],
        // real:usize
    ) -> Result<TS::Value, FreightError> {
        match self.arg_count.max() {
            Some(max) if max > args.len() => {
                return Err(FreightError::IncorrectArgumentCount {
                    expected_min: self.arg_count.min(),
                    expected_max: self.arg_count.max(),
                    actual: args.len(),
                });
            }
            _ => (),
        }
        if self.expressions.is_empty() {
            return Ok(Default::default());
        }

        dbg!(self.arg_count);
        dbg!(&args);
        #[cfg(feature = "variadic_functions")]
        let mut args = match self.arg_count {
            ArgCount::Range { min, max } => match (min, max) {
                (_, Some(_)) => Ok(args),
                (None, None) => Err(vec![TS::Value::gen_list(args.to_vec())]),
                (Some(n), None) => {
                    // if n == args.len() {
                    //     Ok(args)
                    // } else {
                    let mut ret = args[0..n].to_vec();
                    ret.push(TS::Value::gen_list(args[n..].to_vec()));
                    Err(ret)
                    // }
                }
            },
            ArgCount::Fixed(_) => Ok(args),
        };
        #[cfg(feature = "variadic_functions")]
        let args = match &mut args {
            Ok(v) => v,
            Err(v) => &mut v[..],
        };

        dbg!(&args);
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
