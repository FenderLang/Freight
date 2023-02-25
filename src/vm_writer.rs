use crate::{
    execution_engine::{ExecutionEngine, Function},
    expression::{Expression, NativeFunction},
    function::{FunctionRef, FunctionWriter},
    value::Value,
    TypeSystem,
};

#[derive(Debug)]
pub struct VMWriter<TS: TypeSystem> {
    functions: Vec<Function<TS>>,
    globals: usize,
}

impl<TS: TypeSystem> Default for VMWriter<TS> {
    fn default() -> VMWriter<TS> {
        VMWriter::new()
    }
}

impl<TS: TypeSystem> VMWriter<TS> {
    pub fn new() -> VMWriter<TS> {
        Self {
            functions: vec![],
            globals: 0,
        }
    }

    pub fn create_global(&mut self) -> usize {
        self.globals += 1;
        self.globals - 1
    }

    pub fn include_function(&mut self, function: FunctionWriter<TS>) -> FunctionRef<TS> {
        let location = self.functions.len();
        let (arg_count, stack_size) = (function.args, function.stack_size);
        let function_type = function.function_type.clone();
        self.functions.push(function.build());
        FunctionRef {
            arg_count,
            stack_size,
            location,
            function_type,
        }
    }

    pub fn include_native_function<const N: usize>(
        &mut self,
        f: NativeFunction<TS>,
    ) -> FunctionRef<TS> {
        let mut func = FunctionWriter::new(N);
        let args = (0..N)
            .map(|n| Expression::stack(n))
            .collect();
        func.evaluate_expression(Expression::NativeFunctionCall(f, args));
        self.include_function(func)
    }

    pub fn finish(self, entry_point: FunctionRef<TS>) -> ExecutionEngine<TS> {
        ExecutionEngine {
            globals: vec![Value::uninitialized_reference(); self.globals],
            functions: self.functions.into(),
            entry_point: entry_point.location,
            stack_size: entry_point.stack_size,
        }
    }
}
