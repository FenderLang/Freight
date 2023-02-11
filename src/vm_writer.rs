use crate::{
    execution_context::ExecutionContext,
    function::{FunctionBuilder, FunctionRef},
    instruction::Instruction,
    TypeSystem,
};

#[derive(Debug)]
pub struct VMWriter<TS: TypeSystem> {
    instructions: Vec<Instruction<TS>>,
}

impl<TS: TypeSystem> VMWriter<TS> {
    pub fn include_function(&mut self, function: FunctionBuilder<TS>) -> FunctionRef {
        let begin = self.instructions.len();
        let (arg_count, stack_size) = (function.args, function.stack_size);
        self.instructions.extend(function.build_instructions());
        FunctionRef {
            arg_count,
            stack_size,
            location: begin,
        }
    }

    pub fn finish(self) -> ExecutionContext<TS> {
        todo!()
    }
}
