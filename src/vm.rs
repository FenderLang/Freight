use std::ops::Deref;

use crate::{execution_context::{ExecutionContext, RegisterId}, instruction::Instruction, TypeSystem, error::FreightError};

pub struct VM<'a, 'b, TS: TypeSystem> {
    instructions: Vec<Instruction<TS>>,
    pub context: ExecutionContext<'a, TS>,
}

impl<'a, TS: TypeSystem> VM<'a, TS> {
    pub fn new(
        instructions: Vec<Instruction<TS>>,
        stack_size: usize,
        entry_point: usize,
    ) -> VM<'a, TS> {
        Self {
            instructions,
            context: ExecutionContext::new(&[], stack_size, entry_point),
        }
    }

    pub fn stack_size(&self) -> usize {
        self.context.stack_size()
    }

    pub fn run<'b>(&'b mut self) -> Result<&TS::Value, FreightError> {
        self.context.instructions = &*self.instructions;
        while self.context.execute_next()? {}
        Ok(self.context.get_register(RegisterId::Return))
    }
}
