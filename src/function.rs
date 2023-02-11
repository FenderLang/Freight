use crate::{expression_builder::Expression, instruction::Instruction, TypeSystem};

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
            stack_size: 1 + args,
            instructions: vec![],
        }
    }

    pub fn create_variable(&mut self) -> usize {
        self.stack_size += 1;
        self.stack_size
    }

    pub fn assign_value(&mut self, var: usize, expr: Expression<TS>) {
        self.evaluate_expression(expr);
        self.instructions.push(Instruction::MoveFromReturn(var));
    }

    pub fn evaluate_expression(&mut self, expr: Expression<TS>) {
        self.instructions.extend(expr.build_instructions());
    }

    pub fn argument_stack_location(&self, arg: usize) -> usize {
        arg + 1
    }

    pub fn return_expression(&mut self, expr: Expression<TS>) {
        self.evaluate_expression(expr);
        self.instructions.push(Instruction::Return);
    }

    pub fn build_instructions(mut self) -> Vec<Instruction<TS>> {
        let has_return = self.instructions.last().map_or(false, |i| {
            matches!(i, Instruction::Return | Instruction::ReturnConstant(_))
        });
        if !has_return {
            self.instructions.push(Instruction::ReturnConstant(Default::default()));
        }
        self.instructions
    }
}

impl FunctionRef {
    fn arg_count(&self) -> usize {
        self.arg_count
    }

    fn stack_size(&self) -> usize {
        self.stack_size
    }
}