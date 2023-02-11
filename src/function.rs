use crate::{expression::Expression, instruction::Instruction, TypeSystem};

#[derive(Debug)]
pub struct FunctionBuilder<TS: TypeSystem> {
    pub(crate) stack_size: usize,
    pub(crate) args: usize,
    pub(crate) instructions: Vec<Instruction<TS>>,
}

#[derive(Debug, Clone)]
pub struct FunctionRef {
    pub(crate) arg_count: usize,
    pub(crate) stack_size: usize,
    pub(crate) location: usize,
}

impl<TS: TypeSystem> FunctionBuilder<TS> {
    pub fn new(args: usize) -> FunctionBuilder<TS> {
        Self {
            args,
            stack_size: args + 1,
            instructions: vec![],
        }
    }

    pub fn create_variable(&mut self) -> usize {
        let var = self.stack_size;
        self.stack_size += 1;
        var
    }

    pub fn write_instructions(&mut self, instructions: impl IntoIterator<Item = Instruction<TS>>) {
        self.instructions.extend(instructions);
    }

    pub fn assign_value(&mut self, var: usize, expr: Expression<TS>) {
        self.evaluate_expression(expr);
        self.instructions.push(Instruction::MoveFromReturn(var));
    }

    pub fn evaluate_expression(&mut self, expr: Expression<TS>) {
        self.instructions.extend(expr.build_instructions());
    }

    pub fn argument_stack_offset(&self, arg: usize) -> usize {
        arg + 1
    }

    pub fn return_expression(&mut self, expr: Expression<TS>) {
        self.evaluate_expression(expr);
        self.instructions.push(Instruction::Return(self.stack_size));
    }

    pub fn build_instructions(mut self) -> Vec<Instruction<TS>> {
        let has_return = self.instructions.last().map_or(false, |i| {
            matches!(i, Instruction::Return(_) | Instruction::ReturnConstant(_, _))
        });
        if !has_return {
            self.instructions.push(Instruction::ReturnConstant(Default::default(), self.stack_size));
        }
        self.instructions
    }
}

impl FunctionRef {
    pub fn arg_count(&self) -> usize {
        self.arg_count
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size
    }

    pub fn address(&self) -> usize {
        self.location
    }
}