use std::rc::Rc;

use crate::{
    error::FreightError,
    expression::Expression,
    function::{FunctionRef, FunctionType},
    operators::{binary::BinaryOperator, unary::UnaryOperator},
    value::Value,
    TypeSystem,
};

#[derive(Debug)]
pub struct Function<TS: TypeSystem> {
    pub(crate) expressions: Vec<Expression<TS>>,
    pub(crate) stack_size: usize,
    pub(crate) arg_count: usize,
}

pub struct ExecutionEngine<TS: TypeSystem> {
    pub(crate) globals: Vec<TS::Value>,
    pub(crate) functions: Rc<[Function<TS>]>,
    pub(crate) entry_point: usize,
}

impl<TS: TypeSystem> ExecutionEngine<TS> {
    pub fn run(&mut self) -> Result<TS::Value, FreightError> {
        self.functions.clone()[self.entry_point].call(self, vec![])
    }

    pub fn call(
        &mut self,
        func: &FunctionRef<TS>,
        mut args: Vec<TS::Value>,
    ) -> Result<TS::Value, FreightError> {
        if let FunctionType::CapturingRef(captured) = &func.function_type {
            args.splice(0..0, captured.iter().map(|v| v.dupe_ref()));
        }
        self.functions.clone()[func.location].call(self, args)
    }
}

impl<TS: TypeSystem> Function<TS> {
    fn call(
        &self,
        engine: &mut ExecutionEngine<TS>,
        mut args: Vec<TS::Value>,
    ) -> Result<TS::Value, FreightError> {
        if args.len() != self.stack_size {
            return Err(FreightError::IncorrectArgumentCount {
                expected: self.stack_size,
                actual: args.len(),
            });
        }
        while args.len() < self.stack_size {
            args.push(Value::uninitialized_reference());
        }
        if self.expressions.len() == 0 {
            return Ok(Default::default());
        }
        let stack = &mut *args;
        for expr in self.expressions.iter().take(self.expressions.len() - 1) {
            evaluate(expr, engine, stack)?;
        }
        evaluate(self.expressions.last().unwrap(), engine, stack)
    }
}

fn evaluate<TS: TypeSystem>(
    expr: &Expression<TS>,
    engine: &mut ExecutionEngine<TS>,
    stack: &mut [TS::Value],
) -> Result<TS::Value, FreightError> {
    let result = match expr {
        Expression::RawValue(v) => v.clone(),
        Expression::Variable(addr) => stack[*addr].clone(),
        Expression::Global(addr) => engine.globals[*addr].clone(),
        Expression::BinaryOpEval(op, operands) => {
            let [l, r] = &**operands;
            let l = evaluate(l, engine, stack)?;
            let r = evaluate(r, engine, stack)?;
            op.apply_2(&l, &r)
        }
        Expression::UnaryOpEval(op, v) => {
            let v = evaluate(v, engine, stack)?;
            op.apply_1(&v)
        }
        Expression::StaticFunctionCall(func, args) => {
            let collected = args
                .iter()
                .map(|a| evaluate(a, engine, stack))
                .collect::<Result<Vec<_>, _>>()?;
            engine.call(func, collected)?
        }
        Expression::DynamicFunctionCall(func, args) => {
            let func: TS::Value = evaluate(func, engine, stack)?;
            let Some(func): Option<&FunctionRef<TS>> = (&func).cast_to_function() else {
                return Err(FreightError::InvalidInvocationTarget);
            };
            let collected = args
                .iter()
                .map(|a| evaluate(a, engine, stack))
                .collect::<Result<Vec<_>, _>>()?;
            engine.call(func, collected)?
        }
        Expression::FunctionCapture(func) => {
            let FunctionType::CapturingDef(capture) = &func.function_type else {
                return Err(FreightError::InvalidInvocationTarget);
            };
            let mut func = func.clone();
            func.function_type = FunctionType::CapturingRef(
                capture
                    .iter()
                    .map(|i| (&stack[*i]).dupe_ref())
                    .collect::<Vec<_>>()
                    .into(),
            );
            func.into()
        }
        Expression::AssignStack(addr, expr) => {
            let val = evaluate(expr, engine, stack)?;
            stack[*addr].assign(val);
            Default::default()
        }
        Expression::NativeFunctionCall(func, args) => {
            let collected = args
                .iter()
                .map(|a| evaluate(a, engine, stack))
                .collect::<Result<Vec<_>, _>>()?;
            func.0.invoke(engine, collected)?
        }
        Expression::AssignGlobal(addr, expr) => {
            let val = evaluate(expr, engine, stack)?;
            engine.globals[*addr].assign(val);
            Default::default()
        }
    };
    Ok(result)
}

trait Thing {
    const N: usize;
}