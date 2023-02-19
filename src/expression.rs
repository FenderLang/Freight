use crate::{
    error::FreightError,
    execution_context::{Location, RETURN_REGISTER, RIGHT_OPERAND_REGISTER},
    function::FunctionRef,
    instruction::Instruction,
    TypeSystem,
};

#[derive(Debug)]
pub enum Expression<TS: TypeSystem> {
    RawValue(TS::Value),
    Variable(usize),
    Global(usize),
    BinaryOpEval(TS::BinaryOp, Box<Expression<TS>>, Box<Expression<TS>>),
    UnaryOpEval(TS::UnaryOp, Box<Expression<TS>>),
    StaticFunctionCall(FunctionRef<TS>, Vec<Expression<TS>>),
    DynamicFunctionCall(Box<Expression<TS>>, Vec<Expression<TS>>),
    FunctionCapture(FunctionRef<TS>),
}

impl<TS: TypeSystem> Expression<TS> {
    pub fn build(self, dest: Location) -> Result<Vec<Instruction<TS>>, FreightError> {
        let mut vec = vec![];
        build_evaluate(self, &mut vec, dest)?;
        Ok(vec)
    }
}

fn build_evaluate<TS: TypeSystem>(
    expr: Expression<TS>,
    instructions: &mut Vec<Instruction<TS>>,
    dest: Location,
) -> Result<(), FreightError> {
    match expr {
        Expression::RawValue(v) => instructions.push(Instruction::SetRaw {
            location: dest,
            value: v,
        }),
        Expression::Variable(v) => instructions.push(Instruction::Copy {
            from: Location::Stack(v),
            to: dest,
        }),
        Expression::BinaryOpEval(op, l, r) => {
            build_evaluate(*l, instructions, RETURN_REGISTER)?;
            instructions.push(Instruction::Push(RETURN_REGISTER));
            build_evaluate(*r, instructions, RIGHT_OPERAND_REGISTER)?;
            instructions.push(Instruction::Pop(RETURN_REGISTER));
            instructions.push(Instruction::BinaryOperation {
                operator: op,
                left: RETURN_REGISTER,
                right: RIGHT_OPERAND_REGISTER,
                dest,
            });
        }
        Expression::UnaryOpEval(op, x) => {
            build_evaluate(*x, instructions, RETURN_REGISTER)?;
            instructions.push(Instruction::UnaryOperation {
                operator: op,
                operand: RETURN_REGISTER,
                dest,
            })
        }
        Expression::StaticFunctionCall(func, args) => {
            if func.arg_count != args.len() {
                return Err(FreightError::IncorrectArgumentCount {
                    expected: func.arg_count,
                    actual: args.len(),
                });
            }
            build_function_call_args(args, instructions)?;
            instructions.push(Instruction::Invoke {
                arg_count: func.arg_count,
                stack_size: func.stack_size,
                instruction: func.location,
            });
            if dest != RETURN_REGISTER {
                instructions.push(Instruction::Move {
                    from: RETURN_REGISTER,
                    to: dest,
                });
            }
        }
        Expression::DynamicFunctionCall(func, args) => {
            let arg_count = args.len();
            build_function_call_args(args, instructions)?;
            build_evaluate(*func, instructions, RETURN_REGISTER)?;
            instructions.push(Instruction::InvokeDynamic { arg_count });
            if dest != RETURN_REGISTER {
                instructions.push(Instruction::Move {
                    from: RETURN_REGISTER,
                    to: dest,
                });
            }
        }
        Expression::FunctionCapture(func) => {
            instructions.extend([
                Instruction::SetRaw {
                    location: RETURN_REGISTER,
                    value: func.into(),
                },
                Instruction::CaptureValues,
            ]);
        }
        Expression::Global(addr) => instructions.push(Instruction::Copy {
            from: Location::Const(addr),
            to: RETURN_REGISTER,
        }),
    }
    Ok(())
}

fn build_function_call_args<TS: TypeSystem>(
    args: Vec<Expression<TS>>,
    instructions: &mut Vec<Instruction<TS>>,
) -> Result<(), FreightError> {
    for arg in args {
        build_evaluate(arg, instructions, RETURN_REGISTER)?;
        instructions.push(Instruction::Push(RETURN_REGISTER));
    }
    Ok(())
}
