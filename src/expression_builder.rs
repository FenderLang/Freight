use crate::{
    execution_context::{ExecutionContext, HELD_VALUE_LOCATION},
    instruction::Instruction,
    operators::Operator,
    TypeSystem,
};

#[derive(Clone, Debug)]
pub enum Operand<TS: TypeSystem> {
    Function {
        addr: usize,
        args: Vec<Operand<TS>>,
        stack_size: usize,
    },
    ValueRef(usize),
    ValueRaw(TS::Value),
    Expression(Box<ExpressionBuilder<TS>>),
}

pub enum Expression<TS: TypeSystem> {
    UnaryExpression {
        operand: Operand<TS>,
        operator: TS::UnaryOp,
    },
    BinaryExpression {
        operator: TS::BinaryOp,
        right_operand: Operand<TS>,
        left_operand: Operand<TS>,
    },
    SingleElementExpression(Operand<TS>),
}

#[derive(Default, Clone, Debug)]
pub struct ExpressionBuilder<TS: TypeSystem> {
    operator: Option<Operator<TS>>,
    operands: (Option<Operand<TS>>, Option<Operand<TS>>),
}



impl<TS: TypeSystem> ExpressionBuilder<TS> {
    /// Construct a unary expression to either call build on or to be passed to a larger expression.
    pub fn unary_expression(operand: Operand<TS>, operator: TS::UnaryOp) -> ExpressionBuilder<TS> {
        ExpressionBuilder {
            operands: (Some(operand), None),
            operator: Some(Operator::Unary(operator)),
        }
    }

    /// Construct a unary expression to either call build on or to be passed to a larger expression.
    pub fn binary_expression(
        operator: TS::BinaryOp,
        right_operand: Operand<TS>,
        left_operand: Operand<TS>,
    ) -> ExpressionBuilder<TS> {
        ExpressionBuilder {
            operands: (Some(right_operand), Some(left_operand)),
            operator: Some(Operator::Binary(operator)),
        }
    }

    /// Encapsulate a single operand in an expression.
    pub fn single_value_expression(value: Operand<TS>) -> ExpressionBuilder<TS> {
        ExpressionBuilder {
            operator: None,
            operands: (Some(value), None),
        }
    }

    fn expand_function_call_instructions(
        instructions: &mut Vec<Instruction<TS>>,
        function_addr: usize,
        args: Vec<Operand<TS>>,
        stack_size: usize,
    ) {
        let arg_count = args.len();
        for arg in args {
            match arg {
                Operand::Function {
                    addr,
                    args,
                    stack_size,
                } => {
                    ExpressionBuilder::expand_function_call_instructions(
                        instructions,
                        addr,
                        args,
                        stack_size,
                    );
                    instructions.push(Instruction::PushFromReturn);
                }
                Operand::ValueRef(addr) => instructions.push(Instruction::Push(addr)),
                Operand::ValueRaw(val) => instructions.push(Instruction::PushRaw(val)),
                Operand::Expression(builder) => instructions.append(&mut builder.build()),
            }
        }
        instructions.push(Instruction::Invoke(function_addr, arg_count, stack_size));
    }

    pub fn build(self) -> Vec<Instruction<TS>> {
        let mut instructions = Vec::new();

        if let Some(operand) = self.operands.0 {
            match operand {
                Operand::Function {
                    addr,
                    args,
                    stack_size,
                } => {
                    ExpressionBuilder::expand_function_call_instructions(
                        &mut instructions,
                        addr,
                        args,
                        stack_size,
                    );
                    instructions.push(Instruction::MoveFromReturn(HELD_VALUE_LOCATION))
                }
                Operand::ValueRef(addr) => {
                    instructions.push(Instruction::Move(addr, HELD_VALUE_LOCATION))
                }
                Operand::ValueRaw(val) => instructions.push(Instruction::SetHeldRaw(val)),
                Operand::Expression(builder) => {
                    instructions.append(&mut builder.build());
                    instructions.push(Instruction::MoveFromReturn(HELD_VALUE_LOCATION))
                }
            }
        }

        if let Some(operand) = self.operands.1 {
            match operand {
                Operand::Function {
                    addr,
                    args,
                    stack_size,
                } => {
                    ExpressionBuilder::expand_function_call_instructions(
                        &mut instructions,
                        addr,
                        args,
                        stack_size,
                    );
                }
                Operand::Expression(builder) => {
                    instructions.append(&mut builder.build());
                    instructions.push(Instruction::MoveFromReturn(HELD_VALUE_LOCATION))
                }
                Operand::ValueRef(addr) => instructions.push(Instruction::MoveRightOperand(addr)),
                Operand::ValueRaw(val) => instructions.push(Instruction::SetRightOperandRaw(val)),
            }
        }

        match self.operator {
            Some(Operator::Binary(b_op)) => {
                instructions.push(Instruction::BinaryOperationWithHeld(b_op))
            }
            Some(Operator::Unary(u_op)) => {
                instructions.push(Instruction::UnaryOperationWithHeld(u_op))
            }
            None => (),
        }

        instructions
    }
}