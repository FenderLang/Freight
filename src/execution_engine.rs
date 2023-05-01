#[cfg(feature = "variadic_functions")]
use crate::function::ArgCount;
use crate::{
    error::FreightError,
    expression::{Expression, VariableType},
    function::{FunctionRef, FunctionType, FunctionWriter},
    operators::{BinaryOperator, Initializer, UnaryOperator},
    slice_pool::{IntoExactSizeIterator, RcSlicePool},
    value::Value,
    TypeSystem,
};
use crate::{error::OrReturn, function::Function};
use std::cell::UnsafeCell;
use std::rc::Rc;

pub type Stack<'a, T> = &'a mut [T];

pub struct StackPool<T: Default> {
    stack: Vec<T>,
    base: usize,
}

impl<T: Default> StackPool<T> {
    pub fn with_capacity(capacity: usize) -> StackPool<T> {
        let mut stack = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            stack.push(Default::default());
        }
        StackPool { stack, base: 0 }
    }

    pub fn request<'a>(&mut self, capacity: usize) -> &'a mut [T] {
        if self.base + capacity >= self.stack.len() {
            panic!("Stack overflow {} / {}", self.base, self.stack.len());
        }

        unsafe {
            let ptr = self.stack.as_mut_ptr().add(self.base);

            self.base += capacity;
            let slice = std::slice::from_raw_parts_mut(ptr, capacity);
            slice
        }
    }

    pub fn release(&mut self, capacity: usize) {
        self.base -= capacity;
    }
}

impl<T: Default> Default for StackPool<T> {
    fn default() -> Self {
        Self::with_capacity(10000)
    }
}

pub struct ExecutionEngine<TS: TypeSystem> {
    pub(crate) num_globals: usize,
    pub(crate) globals: Vec<TS::Value>,
    pub(crate) functions: UnsafeCell<Vec<Function<TS>>>,
    pub(crate) next_return_target: usize,
    pub(crate) return_value: TS::Value,
    pub stack: StackPool<TS::Value>,
    pub rc_pool: Rc<UnsafeCell<RcSlicePool<TS::Value>>>,
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
            stack: Default::default(),
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
        let mut iter = args.into_exact_size_iter();
        let arg_count = iter.len();
        self.call_internal(func, |_| Ok(iter.next().unwrap()), arg_count)
    }

    pub(crate) fn call_internal(
        &mut self,
        func: &FunctionRef<TS>,
        mut args: impl FnMut(&mut ExecutionEngine<TS>) -> Result<TS::Value, FreightError>,
        arg_count: usize,
    ) -> Result<TS::Value, FreightError> {
        let stack = self.stack.request(func.stack_size);
        if !func.arg_count.valid_arg_count(arg_count) {
            return Err(FreightError::IncorrectArgumentCount {
                expected_min: func.arg_count.min(),
                expected_max: func.arg_count.max(),
                actual: arg_count,
            });
        }
        let mut arg_num = 0;
        let max = func.arg_count.max_capped().min(arg_count);
        while arg_num < max {
            let mut value = args(self)?;
            if func.layout.is_alloc(arg_num) {
                value = value.into_ref();
            } else {
                value = value.clone();
            }
            stack[arg_num] = value;
            arg_num += 1;
        }
        for (i, arg) in (arg_num..).zip(stack[arg_num..].iter_mut()) {
            if func.layout.is_alloc(i) {
                *arg = Value::uninitialized_reference();
            } else {
                *arg = Default::default();
            }
        }

        #[cfg(feature = "variadic_functions")]
        if let ArgCount::Variadic { .. } = func.arg_count {
            let mut vargs = Vec::with_capacity(arg_count - arg_num);
            for _ in arg_num..arg_count {
                vargs.push(args(self)?);
            }
            stack[func.arg_count.max_capped()] = Value::gen_list(vargs);
        }

        if let FunctionType::Native(func) = &func.function_type {
            let value = func(self, stack);
            self.stack.release(stack.len());
            return value;
        }
        let function = self.get_function(func.location);
        let value = match &func.function_type {
            FunctionType::CapturingRef(captures) => function.call(self, stack, captures),
            FunctionType::Static => function.call(self, stack, &[]),
            FunctionType::CapturingDef(_) => Err(FreightError::InvalidInvocationTarget),
            FunctionType::Native(_) => unreachable!("Native function already handled"),
        };
        self.stack.release(stack.len());
        value
    }

    #[inline]
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
                let mut args = args.iter();
                let arg_count = args.len();
                self.call_internal(
                    func,
                    |e| e.evaluate_internal(args.next().unwrap(), stack, captured),
                    arg_count,
                )?
            }
            Expression::DynamicFunctionCall(func, args) => {
                let func: TS::Value = self.evaluate_internal(func, stack, captured)?;
                let Some(func): Option<&FunctionRef<TS>> = func.cast_to_function() else {
                    return Err(FreightError::InvalidInvocationTarget);
                };
                let mut iter = args.iter();
                let arg_count = iter.len();
                self.call_internal(
                    func,
                    |e| e.evaluate_internal(iter.next().unwrap(), stack, captured),
                    arg_count,
                )?
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
                let collected = self.stack.request(args.len());
                for (i, arg) in args.iter().enumerate() {
                    collected[i] = self.evaluate_internal(arg, stack, captured)?.clone();
                }
                let value = func(self, collected)?;
                self.stack.release(collected.len());
                value
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
