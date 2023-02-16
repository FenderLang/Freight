use crate::{
    error::FreightError,
    execution_context::{Location, HELD_VALUE, RETURN_REGISTER, RIGHT_OPERAND_REGISTER},
    function::FunctionRef,
    instruction::Instruction,
    TypeSystem,
};

#[derive(Clone, Debug)]
pub enum Operand<TS: TypeSystem> {
    StaticFunctionCall {
        function: FunctionRef<TS>,
        args: Vec<Operand<TS>>,
    },
    DynamicFunctionCall {
        function: Box<Operand<TS>>,
        args: Vec<Operand<TS>>,
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
    FunctionCapture(FunctionRef<TS>),
}

fn expand_function_args<TS: TypeSystem>(
    instructions: &mut Vec<Instruction<TS>>,
    args: Vec<Operand<TS>>,
) -> Result<(), FreightError> {
    instructions.push(Instruction::PushRaw(Default::default()));
    for arg in args {
        match arg {
            Operand::StaticFunctionCall { function, args } => {
                expand_static_function_call_instructions(instructions, &function, args)?;
                instructions.push(Instruction::Push(RETURN_REGISTER));
            }
            Operand::ValueRef(addr) => instructions.push(Instruction::Push(Location::Addr(addr))),
            Operand::ValueRaw(val) => instructions.push(Instruction::PushRaw(val)),
            Operand::Expression(builder) => instructions.extend(builder.build_instructions()?),
            Operand::DynamicFunctionCall { function, args } => {
                expand_dynamic_function_call_instructions(instructions, *function, args)?;
                instructions.push(Instruction::Push(RETURN_REGISTER));
            }
        }
    }
    Ok(())
}

fn expand_static_function_call_instructions<TS: TypeSystem>(
    instructions: &mut Vec<Instruction<TS>>,
    function: &FunctionRef<TS>,
    args: Vec<Operand<TS>>,
) -> Result<(), FreightError> {
    let arg_count = args.len();
    if arg_count != function.arg_count {
        return Err(FreightError::IncorrectArgumentCount {
            expected: function.arg_count,
            actual: arg_count,
        });
    }
    expand_function_args(instructions, args)?;
    instructions.push(Instruction::Invoke(
        arg_count,
        function.stack_size,
        function.location,
    ));
    Ok(())
}

fn expand_dynamic_function_call_instructions<TS: TypeSystem>(
    instructions: &mut Vec<Instruction<TS>>,
    function: Operand<TS>,
    args: Vec<Operand<TS>>,
) -> Result<(), FreightError> {
    let arg_count = args.len();
    expand_function_args(instructions, args)?;
    expand_first_operand_instructions(function, instructions)?;
    instructions.push(Instruction::InvokeDynamic(arg_count));
    Ok(())
}

fn expand_first_operand_instructions<TS: TypeSystem>(
    operand: Operand<TS>,
    instructions: &mut Vec<Instruction<TS>>,
) -> Result<(), FreightError> {
    match operand {
        Operand::StaticFunctionCall { function, args } => {
            expand_static_function_call_instructions(instructions, &function, args)?;
            instructions.push(Instruction::Move {
                from: RETURN_REGISTER,
                to: HELD_VALUE,
            })
        }
        Operand::ValueRef(addr) => instructions.push(Instruction::Move {
            from: Location::Addr(addr),
            to: HELD_VALUE,
        }),
        Operand::ValueRaw(val) => instructions.push(Instruction::SetRaw {
            location: HELD_VALUE,
            value: val,
        }),
        Operand::Expression(builder) => {
            instructions.append(&mut builder.build_instructions()?);
            instructions.push(Instruction::Move {
                from: RETURN_REGISTER,
                to: HELD_VALUE,
            })
        }
        Operand::DynamicFunctionCall { function, args } => {
            expand_dynamic_function_call_instructions(instructions, *function, args)?;
            instructions.push(Instruction::Move {
                from: RETURN_REGISTER,
                to: HELD_VALUE,
            })
        }
    };
    Ok(())
}

fn expand_second_operand_instructions<TS: TypeSystem>(
    operand: Operand<TS>,
    instructions: &mut Vec<Instruction<TS>>,
) -> Result<(), FreightError> {
    match operand {
        Operand::StaticFunctionCall { function, args } => {
            expand_static_function_call_instructions(instructions, &function, args)?;
        }
        Operand::Expression(builder) => {
            instructions.append(&mut builder.build_instructions()?);
            instructions.push(Instruction::Move {
                from: RETURN_REGISTER,
                to: HELD_VALUE,
            })
        }
        Operand::ValueRef(addr) => instructions.push(Instruction::Move {
            from: Location::Addr(addr),
            to: RIGHT_OPERAND_REGISTER,
        }),
        Operand::ValueRaw(val) => instructions.push(Instruction::SetRaw {
            location: RIGHT_OPERAND_REGISTER,
            value: val,
        }),
        Operand::DynamicFunctionCall { function, args } => {
            expand_dynamic_function_call_instructions(instructions, *function, args)?;
            instructions.push(Instruction::Move {
                from: RETURN_REGISTER,
                to: HELD_VALUE,
            })
        }
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
                instructions.push(Instruction::Move {
                    from: HELD_VALUE,
                    to: RETURN_REGISTER,
                });
            }
            Expression::FunctionCapture(func) => {
                instructions.push(Instruction::SetRaw {
                    location: RETURN_REGISTER,
                    value: func.into(),
                });
                instructions.push(Instruction::CaptureValues);
            }
        }
        Ok(instructions)
    }
}
