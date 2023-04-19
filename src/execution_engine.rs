#[cfg(feature = "variadic_functions")]
use crate::function::ArgCount;
use crate::{
    collection_pool::{IntoExactSizeIterator, PooledVec, RcSlicePool, VecPool},
    error::FreightError,
    expression::{Expression, VariableType},
    function::{FunctionRef, FunctionType, FunctionWriter},
    operators::{BinaryOperator, Initializer, UnaryOperator},
    value::Value,
    TypeSystem,
};
use crate::{error::OrReturn, function::Function};
use std::cell::{RefCell, UnsafeCell};
use std::rc::Rc;

pub struct ExecutionEngine<TS: TypeSystem> {
    pub(crate) num_globals: usize,
    pub(crate) globals: Vec<TS::Value>,
    pub(crate) functions: UnsafeCell<Vec<Function<TS>>>,
    pub(crate) next_return_target: usize,
    pub(crate) return_value: TS::Value,
    pub vec_pool: Rc<RefCell<VecPool<TS::Value>>>,
    pub rc_pool: Rc<RefCell<RcSlicePool<TS::Value>>>,
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
            vec_pool: Default::default(),
            context,
            rc_pool: Default::default(),
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

    #[inline]
    pub fn call(
        &mut self,
        func: &FunctionRef<TS>,
        args: impl IntoExactSizeIterator<Item = TS::Value>,
    ) -> Result<TS::Value, FreightError> {
        let vec = VecPool::from_pool(self.vec_pool.clone(), args);
        self.call_internal(func, vec)
    }

    pub(crate) fn call_internal(
        &mut self,
        func: &FunctionRef<TS>,
        mut args: PooledVec<TS::Value>,
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
        if let FunctionType::Native(func) = &func.function_type {
            return func(self, args);
        }
        let function = self.get_function(func.location);
        match &func.function_type {
            FunctionType::CapturingRef(captures) => function.call(self, &mut args, captures),
            FunctionType::Static => function.call(self, &mut args, &[]),
            FunctionType::CapturingDef(_) => Err(FreightError::InvalidInvocationTarget),
            FunctionType::Native(_) => unreachable!("Native function already handled"),
        }
    }

    pub fn evaluate(&mut self, expr: &Expression<TS>) -> Result<TS::Value, FreightError> {
        self.evaluate_internal(expr, &mut [], &[])
    }

    pub(crate) fn evaluate_internal(
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
                let l = self.evaluate_internal(l, stack, captured)?;
                let r = self.evaluate_internal(r, stack, captured)?;
                op.apply_2(&l, &r)
            }
            Expression::UnaryOpEval(op, v) => {
                let v = self.evaluate_internal(v, stack, captured)?;
                op.apply_1(&v)
            }
            Expression::StaticFunctionCall(func, args) => {
                let mut collected = VecPool::request(self.vec_pool.clone(), func.stack_size);
                for arg in args {
                    collected.push(
                        self.evaluate_internal(arg, stack, captured)?
                            .clone()
                            .into_ref(),
                    );
                }
                self.call_internal(func, collected)?
            }
            Expression::DynamicFunctionCall(func, args) => {
                let func: TS::Value = self.evaluate_internal(func, stack, captured)?;
                let Some(func): Option<&FunctionRef<TS>> = func.cast_to_function() else {
                return Err(FreightError::InvalidInvocationTarget);
            };
                let mut collected = VecPool::request(self.vec_pool.clone(), func.stack_size);
                for arg in args {
                    collected.push(
                        self.evaluate_internal(arg, stack, captured)?
                            .clone()
                            .into_ref(),
                    );
                }
                self.call_internal(func, collected)?
            }
            Expression::FunctionCapture(func) => {
                let FunctionType::CapturingDef(capture) = &func.function_type else {
                return Err(FreightError::InvalidInvocationTarget);
            };
                let mut func = func.clone();
                let captures_iter = capture.iter().map(|var| match var {
                    VariableType::Captured(addr) => captured[*addr].dupe_ref(),
                    VariableType::Stack(addr) => stack[*addr].dupe_ref(),
                    VariableType::Global(addr) => self.globals[*addr].dupe_ref(),
                });

                func.function_type = FunctionType::CapturingRef(RcSlicePool::from_pool(
                    self.rc_pool.clone(),
                    captures_iter,
                ));
                func.into()
            }
            Expression::AssignStack(addr, expr) => {
                let val = self.evaluate_internal(expr, stack, captured)?;
                stack[*addr].assign(val);
                Default::default()
            }
            Expression::NativeFunctionCall(func, args) => {
                let mut collected = VecPool::request(self.vec_pool.clone(), args.len());
                for arg in args {
                    collected.push(self.evaluate_internal(arg, stack, captured)?.clone());
                }
                func(self, collected)?
            }
            Expression::AssignGlobal(addr, expr) => {
                let val = self.evaluate_internal(expr, stack, captured)?;
                self.globals[*addr].assign(val);
                Default::default()
            }
            Expression::AssignDynamic(args) => {
                let [target, value] = &**args;
                let mut target = self.evaluate_internal(target, stack, captured)?.dupe_ref();
                let value = self.evaluate_internal(value, stack, captured)?;
                target.assign(value);
                Default::default()
            }
            Expression::Initialize(init, args) => {
                let mut collected = Vec::with_capacity(args.len());
                for arg in args {
                    collected.push(self.evaluate_internal(arg, stack, captured)?);
                }
                init.initialize(collected, self)
            }
            Expression::ReturnTarget(target, expr) => self
                .evaluate_internal(&**expr, stack, captured)
                .or_return(*target, self)?,
            Expression::Return(target, expr) => {
                self.return_value = self.evaluate_internal(&**expr, stack, captured)?;
                return Err(FreightError::Return { target: *target });
            }
        };
        Ok(result)
    }
}
