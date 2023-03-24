use crate::{
    execution_engine::{ExecutionEngine, Function},
    expression::{Expression, NativeFunction},
    function::{FunctionRef, FunctionWriter},
    TypeSystem,
};

#[derive(Debug)]
pub struct VMWriter<TS: TypeSystem> {
    functions: Vec<Function<TS>>,
    globals: usize,
    next_return_target: usize,
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
            next_return_target: 0,
        }
    }

    /// Create a new global variable and return its address
    pub fn create_global(&mut self) -> usize {
        self.globals += 1;
        self.globals - 1
    }

    pub fn create_return_target(&mut self) -> usize {
        self.next_return_target += 1;
        self.next_return_target - 1
    }

    /// Include a function in the VM and return a reference to it
    pub fn include_function(
        &mut self,
        function: FunctionWriter<TS>,
        return_target: usize,
    ) -> FunctionRef<TS> {
        let location = self.functions.len();
        let (arg_count, stack_size) = (function.args, function.stack_size);
        let function_type = function.function_type.clone();
        self.functions.push(function.build(return_target));
        FunctionRef {
            arg_count,
            stack_size,
            location,
            function_type,
        }
    }

    /// Create a wrapper for a native function and return a reference to it
    pub fn include_native_function(
        &mut self,
        f: NativeFunction<TS>,
        args: usize,
    ) -> FunctionRef<TS> {
        let mut func = FunctionWriter::new(args);
        let args = (0..args).map(|n| Expression::stack(n)).collect();
        func.evaluate_expression(Expression::NativeFunctionCall(f, args));
        let return_target = self.create_return_target();
        self.include_function(func, return_target)
    }

    /// Build an [ExecutionEngine] with the given function as an entry point
    pub fn finish(
        self,
        entry_point: FunctionRef<TS>,
        context: TS::GlobalContext,
    ) -> ExecutionEngine<TS> {
        ExecutionEngine {
            num_globals: self.globals,
            globals: vec![],
            functions: self.functions.into(),
            entry_point: entry_point.location,
            stack_size: entry_point.stack_size,
            return_value: Default::default(),
            context,
        }
    }

    pub fn finish_default(self, entry_point: FunctionRef<TS>) -> ExecutionEngine<TS>
    where
        TS::GlobalContext: Default,
    {
        ExecutionEngine {
            num_globals: self.globals,
            globals: vec![],
            functions: self.functions.into(),
            entry_point: entry_point.location,
            stack_size: entry_point.stack_size,
            return_value: Default::default(),
            context: Default::default(),
        }
    }
}
