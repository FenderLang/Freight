#[cfg(feature = "variadic_functions")]
use crate::function::ArgCount;
use crate::{
    error::FreightError,
    expression::{Expression, VariableType},
    function::{FunctionRef, FunctionType, FunctionWriter},
    operators::{BinaryOperator, Initializer, UnaryOperator},
    value::Value,
    TypeSystem,
};
use crate::{error::OrReturn, function::Function};
use std::cell::UnsafeCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct ExecutionEngine<TS: TypeSystem> {
    pub(crate) num_globals: usize,
    pub(crate) globals: Vec<TS::Value>,
    pub(crate) functions: UnsafeCell<Vec<Function<TS>>>,
    pub(crate) next_return_target: usize,
    pub(crate) return_value: TS::Value,
    pub context: TS::GlobalContext,
}

impl<TS: TypeSystem> ExecutionEngine<TS> {
    pub fn new(context: TS::GlobalContext) -> Self {
        Self {
            num_globals: 0,
            globals: vec![],
            functions: vec![].into(),
            next_return_target: 0,
            return_value: Default::default(),
            context,
        }
    }

    pub fn new_default() -> Self
    where
        TS::GlobalContext: Default,
    {
        Self::new(Default::default())
    }

    #[inline]
    pub fn get_function<'a>(&self, id: usize) -> &'a Function<TS> {
        unsafe { &(*self.functions.get())[id] }
    }

    pub fn register_function(
        &mut self,
        func: FunctionWriter<TS>,
        return_target: usize,
    ) -> FunctionRef<TS> {
        unsafe {
            let functions = &mut *self.functions.get();
            let func_ref = func.to_ref(functions.len());
            let func = func.build(return_target);
            functions.push(func);
            func_ref
        }
    }

    pub fn create_return_target(&mut self) -> usize {
        self.next_return_target += 1;
        self.next_return_target - 1
    }

    pub fn create_global(&mut self) -> usize {
        self.globals.push(Value::uninitialized_reference());
        self.globals.len() - 1
    }

    pub fn reset_globals(&mut self) {
        self.globals = vec![Value::uninitialized_reference(); self.num_globals];
    }

    pub fn call(
        &mut self,
        func: &FunctionRef<TS>,
        mut args: Vec<TS::Value>,
    ) -> Result<TS::Value, FreightError> {
        if !func.arg_count.valid_arg_count(args.len()) {
            return Err(FreightError::IncorrectArgumentCount {
                expected_min: func.arg_count.min(),
                expected_max: func.arg_count.max(),
                actual: args.len(),
            });
        }

        while args.len() < func.arg_count.max_capped() {
            args.push(Value::uninitialized_reference());
        }

        #[cfg(feature = "variadic_functions")]
        if let ArgCount::Variadic { min: _, max } = func.arg_count {
            let vargs = args.split_off(max);
            args.push(crate::value::Value::gen_list(vargs));
        }

        for _ in 0..func.variable_count {
            args.push(Value::uninitialized_reference());
        }
        let function = self.get_function(func.location);
        match &func.function_type {
            FunctionType::CapturingRef(captures) => function.call(self, &mut args, captures),
            FunctionType::Static => function.call(self, &mut args, &[]),
            FunctionType::CapturingDef(_) => Err(FreightError::InvalidInvocationTarget),
            FunctionType::Native(func) => func(self, args),
        }
    }

    pub fn evaluate(
        &mut self,
        expr: &Expression<TS>,
        stack: &mut [TS::Value],
        captured: &[TS::Value],
    ) -> Result<TS::Value, FreightError> {
        let result = match expr {
            Expression::RawValue(v) => v.clone(),
            Expression::Variable(var) => match var {
                VariableType::Captured(addr) => captured[*addr].dupe_ref(),
                VariableType::Stack(addr) => stack[*addr].dupe_ref(),
                VariableType::Global(addr) => self.globals[*addr].dupe_ref(),
            },
            Expression::BinaryOpEval(op, operands) => {
                let [l, r] = &**operands;
                let l = self.evaluate(l, stack, captured)?;
                let r = self.evaluate(r, stack, captured)?;
                op.apply_2(&l, &r)
            }
            Expression::UnaryOpEval(op, v) => {
                let v = self.evaluate(v, stack, captured)?;
                op.apply_1(&v)
            }
            Expression::StaticFunctionCall(func, args) => {
                let mut collected = Vec::with_capacity(func.stack_size);
                for arg in args {
                    collected.push(self.evaluate(arg, stack, captured)?.clone().into_ref());
                }
                self.call(func, collected)?
            }
            Expression::DynamicFunctionCall(func, args) => {
                let func: TS::Value = self.evaluate(func, stack, captured)?;
                let Some(func): Option<&FunctionRef<TS>> = func.cast_to_function() else {
                return Err(FreightError::InvalidInvocationTarget);
            };
                let mut collected = Vec::with_capacity(func.stack_size);
                for arg in args {
                    collected.push(self.evaluate(arg, stack, captured)?.clone().into_ref());
                }
                self.call(func, collected)?
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
                            VariableType::Global(addr) => self.globals[*addr].dupe_ref(),
                        })
                        .collect::<Rc<[_]>>(),
                );
                func.into()
            }
            Expression::AssignStack(addr, expr) => {
                let val = self.evaluate(expr, stack, captured)?;
                stack[*addr].assign(val);
                Default::default()
            }
            Expression::NativeFunctionCall(func, args) => {
                let mut collected = Vec::with_capacity(args.len());
                for arg in args {
                    collected.push(self.evaluate(arg, stack, captured)?.clone());
                }
                func(self, collected)?
            }
            Expression::AssignGlobal(addr, expr) => {
                let val = self.evaluate(expr, stack, captured)?;
                self.globals[*addr].assign(val);
                Default::default()
            }
            Expression::AssignDynamic(args) => {
                let [target, value] = &**args;
                let mut target = self.evaluate(target, stack, captured)?.dupe_ref();
                let value = self.evaluate(value, stack, captured)?;
                target.assign(value);
                Default::default()
            }
            Expression::Initialize(init, args) => {
                let mut collected = Vec::with_capacity(args.len());
                for arg in args {
                    collected.push(self.evaluate(arg, stack, captured)?);
                }
                init.initialize(collected, self)
            }
            Expression::ReturnTarget(target, expr) => self
                .evaluate(&**expr, stack, captured)
                .or_return(*target, self)?,
            Expression::Return(target, expr) => {
                self.return_value = self.evaluate(&**expr, stack, captured)?;
                return Err(FreightError::Return { target: *target });
            }
        };
        Ok(result)
    }
}
