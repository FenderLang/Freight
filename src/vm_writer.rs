use crate::{
    execution_context::ExecutionContext,
    expression::Expression,
    function::{FunctionBuilder, FunctionRef},
    instruction::Instruction,
    TypeSystem, error::FreightError,
};

#[derive(Debug)]
pub struct VMWriter<TS: TypeSystem> {
    instructions: Vec<Instruction<TS>>,
    stack_size: usize,
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
            stack_size: 1,
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

    pub fn include_function(&mut self, function: FunctionBuilder<TS>) -> FunctionRef<TS> {
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

    pub fn declare_variable(&mut self) -> usize {
        self.stack_size += 1;
        self.stack_size - 1
    }

    pub fn evaluate_expression(&mut self, expression: Expression<TS>) -> Result<usize, FreightError> {
        Ok(self.write_instructions(expression.build_instructions()?))
    }

    pub fn finish(self, entry_point: usize) -> ExecutionContext<TS> {
        ExecutionContext::new(self.instructions, self.stack_size, entry_point)
    }
}
