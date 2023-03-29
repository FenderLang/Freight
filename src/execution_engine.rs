use crate::{
    error::FreightError,
    expression::{Expression, VariableType},
    function::{FunctionRef, FunctionType},
    operators::{BinaryOperator, Initializer, UnaryOperator},
    value::Value,
    TypeSystem,
};
use crate::{error::OrReturn, function::Function};
use std::rc::Rc;

#[derive(Debug)]
pub struct ExecutionEngine<TS: TypeSystem> {
    pub(crate) num_globals: usize,
    pub(crate) globals: Vec<TS::Value>,
    pub(crate) functions: Rc<[Function<TS>]>,
    pub(crate) entry_point: usize,
    pub(crate) stack_size: usize,
    pub(crate) return_value: TS::Value,
    pub context: TS::GlobalContext,
}

impl<TS: TypeSystem> ExecutionEngine<TS> {
    /// Run the VM
    pub fn run(&mut self) -> Result<TS::Value, FreightError> {
        self.globals = vec![Value::uninitialized_reference(); self.num_globals];
        self.functions.clone()[self.entry_point].call(
            self,
            &mut vec![Value::uninitialized_reference(); self.stack_size],
            &[],
        )
    }

    pub fn call(
        &mut self,
        func: &FunctionRef<TS>,
        mut args: Vec<TS::Value>,
    ) -> Result<TS::Value, FreightError> {
        while args.len() < func.stack_size - if func.arg_count.max().is_none() { 1 } else { 0 } {
            args.push(Value::uninitialized_reference());
        }
        if let FunctionType::CapturingRef(captures) = &func.function_type {
            self.functions.clone()[func.location].call(self, &mut args, captures)
        } else {
            self.functions.clone()[func.location].call(self, &mut args, &[])
        }
    }
}

pub fn evaluate<TS: TypeSystem>(
    expr: &Expression<TS>,
    engine: &mut ExecutionEngine<TS>,
    stack: &mut [TS::Value],
    captured: &[TS::Value],
) -> Result<TS::Value, FreightError> {
    let result = match expr {
        Expression::RawValue(v) => v.clone(),
        Expression::Variable(var) => match var {
            VariableType::Captured(addr) => captured[*addr].dupe_ref(),
            VariableType::Stack(addr) => stack[*addr].dupe_ref(),
            VariableType::Global(addr) => engine.globals[*addr].dupe_ref(),
        },
        Expression::BinaryOpEval(op, operands) => {
            let [l, r] = &**operands;
            let l = evaluate(l, engine, stack, captured)?;
            let r = evaluate(r, engine, stack, captured)?;
            op.apply_2(&l, &r)
        }
        Expression::UnaryOpEval(op, v) => {
            let v = evaluate(v, engine, stack, captured)?;
            op.apply_1(&v)
        }
        Expression::StaticFunctionCall(func, args) => {
            let mut collected = Vec::with_capacity(func.stack_size);
            for arg in args {
                collected.push(evaluate(arg, engine, stack, captured)?.clone().into_ref());
            }
            engine.call(func, collected)?
        }
        Expression::DynamicFunctionCall(func, args) => {
            let func: TS::Value = evaluate(func, engine, stack, captured)?;
            let Some(func): Option<&FunctionRef<TS>> = func.cast_to_function() else {
                return Err(FreightError::InvalidInvocationTarget);
            };
            let mut collected = Vec::with_capacity(func.stack_size);
            for arg in args {
                collected.push(evaluate(arg, engine, stack, captured)?.clone().into_ref());
            }
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
                    .map(|var| match var {
                        VariableType::Captured(addr) => captured[*addr].dupe_ref(),
                        VariableType::Stack(addr) => stack[*addr].dupe_ref(),
                        VariableType::Global(addr) => engine.globals[*addr].dupe_ref(),
                    })
                    .collect::<Rc<[_]>>(),
            );
            func.into()
        }
        Expression::AssignStack(addr, expr) => {
            let val = evaluate(expr, engine, stack, captured)?;
            stack[*addr].assign(val);
            Default::default()
        }
        Expression::NativeFunctionCall(func, args) => {
            let mut collected = Vec::with_capacity(args.len());
            for arg in args {
                collected.push(evaluate(arg, engine, stack, captured)?.clone());
            }
            func(engine, collected)?
        }
        Expression::AssignGlobal(addr, expr) => {
            let val = evaluate(expr, engine, stack, captured)?;
            engine.globals[*addr].assign(val);
            Default::default()
        }
        Expression::AssignDynamic(args) => {
            let [target, value] = &**args;
            let mut target = evaluate(target, engine, stack, captured)?.dupe_ref();
            let value = evaluate(value, engine, stack, captured)?;
            target.assign(value);
            Default::default()
        }
        Expression::Initialize(init, args) => {
            let mut collected = Vec::with_capacity(args.len());
            for arg in args {
                collected.push(evaluate(arg, engine, stack, captured)?);
            }
            init.initialize(collected)
        }
        Expression::ReturnTarget(target, expr) => {
            evaluate(&**expr, engine, stack, captured).or_return(*target, engine)?
        }
        Expression::Return(target, expr) => {
            engine.return_value = evaluate(&**expr, engine, stack, captured)?;
            return Err(FreightError::Return { target: *target });
        }
    };
    Ok(result)
}
