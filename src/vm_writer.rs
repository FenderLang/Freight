use std::rc::Rc;

use crate::{
    execution_context::ExecutionContext,
    function::{FunctionRef, FunctionWriter},
    instruction::Instruction,
    TypeSystem,
};

#[derive(Debug)]
pub struct VMWriter<TS: TypeSystem> {
    instructions: Vec<Instruction<TS>>,
}

impl<TS: TypeSystem> Default for VMWriter<TS> {
    fn default() -> VMWriter<TS> {
        VMWriter::new()
    }
}

impl<TS: TypeSystem> VMWriter<TS> {
    pub fn new() -> VMWriter<TS> {
        Self {
            instructions: vec![],
        }
    }

    pub fn write_instructions(
        &mut self,
        instructions: impl IntoIterator<Item = Instruction<TS>>,
    ) -> usize {
        let begin = self.instructions.len();
        self.instructions.extend(instructions);
        begin
    }

    pub fn include_function(&mut self, function: FunctionWriter<TS>) -> FunctionRef<TS> {
        let begin = self.instructions.len();
        let (arg_count, stack_size) = (function.args, function.stack_size);
        let function_type = function.function_type.clone();
        self.instructions.extend(function.build_instructions());
        FunctionRef {
            arg_count,
            stack_size,
            location: begin,
            function_type,
        }
    }

    pub fn include_native_function<const N: usize>(
        &mut self,
        f: fn(&mut ExecutionContext<TS>, [TS::Value; N]) -> TS::Value,
    ) -> FunctionRef<TS> {
        let mut func = FunctionWriter::new(N);
        func.write_instructions([
            Instruction::InvokeNative {
                function: Rc::new(f),
                arg_count: N,
            },
            Instruction::Return { stack_size: 1 },
        ]);
        self.include_function(func)
    }

    pub fn finish(self, entry_point: FunctionRef<TS>) -> ExecutionContext<TS> {
        ExecutionContext::new(
            self.instructions,
            entry_point.stack_size,
            entry_point.location,
        )
    }
}
