use crate::{
    error::FreightError,
    execution_context::{Location, HELD_VALUE, RETURN_REGISTER},
    function::FunctionRef,
    instruction::Instruction,
    value::Value,
    TypeSystem,
};

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
    pub fn build(self) -> Result<Vec<Instruction<TS>>, FreightError> {
        let mut vec = vec![];
        build_evaluate(self, &mut vec)?;
        Ok(vec)
    }
}

fn build_evaluate<TS: TypeSystem>(
    expr: Expression<TS>,
    instructions: &mut Vec<Instruction<TS>>,
) -> Result<(), FreightError> {
    match expr {
        Expression::RawValue(v) => instructions.push(Instruction::SetRaw {
            location: RETURN_REGISTER,
            value: v,
        }),
        Expression::Variable(v) => instructions.push(Instruction::Copy {
            from: Location::Stack(v),
            to: RETURN_REGISTER,
        }),
        Expression::BinaryOpEval(op, l, r) => {
            build_evaluate(*l, instructions)?;
            instructions.push(Instruction::Move {
                from: RETURN_REGISTER,
                to: HELD_VALUE,
            });
            build_evaluate(*r, instructions)?;
            instructions.push(Instruction::BinaryOperation {
                operator: op,
                left: HELD_VALUE,
                right: RETURN_REGISTER,
            });
        }
        Expression::UnaryOpEval(op, x) => {
            build_evaluate(*x, instructions)?;
            instructions.push(Instruction::UnaryOperation {
                operator: op,
                operand: RETURN_REGISTER,
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
            })
        }
        Expression::DynamicFunctionCall(func, args) => {
            let arg_count = args.len();
            build_function_call_args(args, instructions)?;
            build_evaluate(*func, instructions)?;
            instructions.push(Instruction::InvokeDynamic { arg_count });
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
    instructions.push(Instruction::PushRaw(Value::uninitialized_reference()));
    for arg in args {
        build_evaluate(arg, instructions)?;
        instructions.push(Instruction::Push(RETURN_REGISTER));
    }
    Ok(())
}
