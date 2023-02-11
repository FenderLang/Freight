use crate::{
    error::FreightError, execution_context::HELD_VALUE_LOCATION, function::FunctionRef,
    instruction::Instruction, TypeSystem,
};

#[derive(Clone, Debug)]
pub enum Operand<TS: TypeSystem> {
    Function {
        args: Vec<Operand<TS>>,
        function: FunctionRef,
    },
    ValueRef(usize),
    ValueRaw(TS::Value),
    Expression(Box<Expression<TS>>),
}

#[derive(Clone, Debug)]
pub enum Expression<TS: TypeSystem> {
    /// Construct a unary operation expression. This can be passed to a larger expression or call `build()` to turn into a list of `Instruction`s.
    UnaryOpEval {
        operand: Operand<TS>,
        operator: TS::UnaryOp,
    },
    /// Construct a unary Binary expression. This can be passed to a larger expression or call `build()` to turn into a list of `Instruction`s.
    BinaryOpEval {
        operator: TS::BinaryOp,
        right_operand: Operand<TS>,
        left_operand: Operand<TS>,
    },
    /// Encapsulate a single operand in an expression
    Eval(Operand<TS>),
}

fn expand_function_call_instructions<TS: TypeSystem>(
    instructions: &mut Vec<Instruction<TS>>,
    function: &FunctionRef,
    args: Vec<Operand<TS>>,
) -> Result<(), FreightError> {
    instructions.push(Instruction::PushRaw(Default::default()));
    let arg_count = args.len();
    if arg_count != function.arg_count {
        return Err(FreightError::IncorrectArgumentCount {
            expected: function.arg_count,
            actual: arg_count,
        });
    }
    for arg in args {
        match arg {
            Operand::Function { function, args } => {
                expand_function_call_instructions(instructions, &function, args)?;
                instructions.push(Instruction::PushFromReturn);
            }
            Operand::ValueRef(addr) => instructions.push(Instruction::Push(addr)),
            Operand::ValueRaw(val) => instructions.push(Instruction::PushRaw(val)),
            Operand::Expression(builder) => instructions.append(&mut builder.build_instructions()?),
        }
    }
    instructions.push(Instruction::Invoke(
        arg_count,
        function.stack_size,
        function.location,
    ));
    Ok(())
}

fn expand_first_operand_instructions<TS: TypeSystem>(
    operand: Operand<TS>,
    instructions: &mut Vec<Instruction<TS>>,
) -> Result<(), FreightError> {
    match operand {
        Operand::Function { function, args } => {
            expand_function_call_instructions(instructions, &function, args)?;
            instructions.push(Instruction::MoveFromReturn(HELD_VALUE_LOCATION))
        }
        Operand::ValueRef(addr) => instructions.push(Instruction::Move(addr, HELD_VALUE_LOCATION)),
        Operand::ValueRaw(val) => instructions.push(Instruction::SetHeldRaw(val)),
        Operand::Expression(builder) => {
            instructions.append(&mut builder.build_instructions()?);
            instructions.push(Instruction::MoveFromReturn(HELD_VALUE_LOCATION))
        }
    };
    Ok(())
}

fn expand_second_operand_instructions<TS: TypeSystem>(
    operand: Operand<TS>,
    instructions: &mut Vec<Instruction<TS>>,
) -> Result<(), FreightError> {
    match operand {
        Operand::Function { function, args } => {
            expand_function_call_instructions(instructions, &function, args)?;
        }
        Operand::Expression(builder) => {
            instructions.append(&mut builder.build_instructions()?);
            instructions.push(Instruction::MoveFromReturn(HELD_VALUE_LOCATION))
        }
        Operand::ValueRef(addr) => instructions.push(Instruction::MoveRightOperand(addr)),
        Operand::ValueRaw(val) => instructions.push(Instruction::SetRightOperandRaw(val)),
    }
    Ok(())
}

impl<TS: TypeSystem> Expression<TS> {
    pub fn build_instructions(self) -> Result<Vec<Instruction<TS>>, FreightError> {
        let mut instructions = Vec::new();

        match self {
            Expression::UnaryOpEval { operand, operator } => {
                expand_first_operand_instructions(operand, &mut instructions)?;
                instructions.push(Instruction::UnaryOperation(operator));
            }
            Expression::BinaryOpEval {
                operator,
                right_operand,
                left_operand,
            } => {
                expand_first_operand_instructions(left_operand, &mut instructions)?;
                expand_second_operand_instructions(right_operand, &mut instructions)?;
                instructions.push(Instruction::BinaryOperationWithHeld(operator));
            }
            Expression::Eval(operand) => {
                expand_first_operand_instructions(operand, &mut instructions)?;
                instructions.push(Instruction::MoveToReturn(HELD_VALUE_LOCATION));
            }
        }
        Ok(instructions)
    }
}
