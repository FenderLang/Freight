use crate::{expression::Expression, instruction::Instruction, TypeSystem, error::FreightError};

#[derive(Debug)]
pub struct FunctionBuilder<TS: TypeSystem> {
    pub(crate) stack_size: usize,
    pub(crate) args: usize,
    pub(crate) instructions: Vec<Instruction<TS>>,
    pub(crate) function_type: FunctionType<TS>,
}

#[derive(Debug, Clone)]
pub enum FunctionType<TS: TypeSystem> {
    /// A static reference to a function, which can't capture any values
    Static,
    /// A reference to a function which captures values, but hasn't been initialized with those values
    CapturingDef(Vec<usize>),
    /// A reference to a function which captures values bundled with those captured values
    CapturingRef(Vec<TS::Value>),
}

#[derive(Debug, Clone)]
pub struct FunctionRef<TS: TypeSystem> {
    pub(crate) arg_count: usize,
    pub(crate) stack_size: usize,
    pub(crate) location: usize,
    pub(crate) function_type: FunctionType<TS>,
}

impl<TS: TypeSystem> FunctionBuilder<TS> {
    pub fn new(args: usize) -> FunctionBuilder<TS> {
        Self {
            args,
            stack_size: args + 1,
            instructions: vec![],
            function_type: FunctionType::Static,
        }
    }

    pub fn new_capturing(args: usize, capture: Vec<usize>) -> FunctionBuilder<TS> {
        Self {
            args,
            stack_size: args + capture.len() + 1,
            instructions: vec![],
            function_type: FunctionType::CapturingDef(capture),
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

    pub fn assign_value(&mut self, var: usize, expr: Expression<TS>) -> Result<(), FreightError> {
        self.evaluate_expression(expr)?;
        self.instructions.push(Instruction::MoveFromReturn(var));
        Ok(())
    }

    pub fn evaluate_expression(&mut self, expr: Expression<TS>) -> Result<(), FreightError> {
        self.instructions.extend(expr.build_instructions()?);
        Ok(())
    }

    pub fn argument_stack_offset(&self, arg: usize) -> usize {
        arg + 1
    }

    pub fn return_expression(&mut self, expr: Expression<TS>) -> Result<(), FreightError> {
        self.evaluate_expression(expr)?;
        self.instructions.push(Instruction::Return(self.stack_size));
        Ok(())
    }

    pub fn build_instructions(mut self) -> Vec<Instruction<TS>> {
        let has_return = self.instructions.last().map_or(false, |i| {
            matches!(
                i,
                Instruction::Return(_) | Instruction::ReturnConstant(_, _)
            )
        });
        if !has_return {
            self.instructions.push(Instruction::ReturnConstant(
                Default::default(),
                self.stack_size,
            ));
        }
        self.instructions
    }
}

impl<TS: TypeSystem> FunctionRef<TS> {
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

impl<TS: TypeSystem> FunctionType<TS> {
    pub fn can_invoke(&self) -> bool {
        use FunctionType::*;
        match self {
            Static | CapturingRef(_) => true,
            CapturingDef(_) => false,
        }
    }
}