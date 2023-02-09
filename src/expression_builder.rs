use crate::{
    execution_context::{ExecutionContext, HELD_VALUE_LOCATION},
    instruction::Instruction,
    operators::Operator,
    TypeSystem,
};

#[derive(Clone)]
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

#[derive(Default, Clone)]
pub struct ExpressionBuilder<TS: TypeSystem> {
    operator: Option<Operator<TS>>,
    operands: (Option<Operand<TS>>, Option<Operand<TS>>),
}

impl<TS: TypeSystem> ExpressionBuilder<TS> {
    pub fn new() -> ExpressionBuilder<TS> {
        ExpressionBuilder {
            operator: None,
            operands: (None, None),
        }
    }

    pub fn set_value(&mut self, value: Operand<TS>) -> &mut ExpressionBuilder<TS> {
        match &self.operands {
            (None, _) => self.operands.0 = Some(value),
            (Some(_), None) => self.operands.1 = Some(value),
            _ => (),
        }
        self
    }

    pub fn set_left_operand(&mut self, value: Operand<TS>) -> &mut ExpressionBuilder<TS> {
        self.operands.0 = Some(value);
        self
    }

    pub fn set_right_operand(&mut self, value: Operand<TS>) -> &mut ExpressionBuilder<TS> {
        self.operands.1 = Some(value);
        self
    }

    pub fn set_operator(&mut self, operator: Operator<TS>) -> &mut ExpressionBuilder<TS> {
        self.operator = Some(operator);
        self
    }

    fn build_function(
        instructions: &mut Vec<Instruction<TS>>,
        execution_context: &mut ExecutionContext<TS>,
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
                    ExpressionBuilder::build_function(
                        instructions,
                        execution_context,
                        addr,
                        args,
                        stack_size,
                    );
                    instructions.push(Instruction::PushFromReturn);
                }
                Operand::ValueRef(addr) => instructions.push(Instruction::Push(addr)),
                Operand::ValueRaw(val) => instructions.push(Instruction::PushRaw(val)),
                Operand::Expression(builder) => {
                    instructions.append(&mut builder.build(execution_context))
                }
            }
        }
        instructions.push(Instruction::Invoke(function_addr, arg_count, stack_size));
    }

    pub fn build(self, execution_context: &mut ExecutionContext<TS>) -> Vec<Instruction<TS>> {
        let mut instructions = Vec::new();

        if let Some(operand) = self.operands.0 {
            match operand {
                Operand::Function {
                    addr,
                    args,
                    stack_size,
                } => {
                    ExpressionBuilder::build_function(
                        &mut instructions,
                        execution_context,
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
                    builder.build(execution_context);
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
                    ExpressionBuilder::build_function(
                        &mut instructions,
                        execution_context,
                        addr,
                        args,
                        stack_size,
                    );
                }
                Operand::Expression(builder) => {
                    instructions.append(&mut builder.build(execution_context));
                    instructions.push(Instruction::MoveFromReturn(HELD_VALUE_LOCATION))
                }
                Operand::ValueRef(addr) => instructions.push(Instruction::MoveRightOperand(addr)),
                Operand::ValueRaw(val) => instructions.push(Instruction::SetRightOperandRaw(val)),
            }
        }

        if let Some(op) = self.operator {
            match op {
                Operator::Binary(b_op) => {
                    instructions.push(Instruction::BinaryOperationWithHeld(b_op))
                }
                Operator::Unary(u_op) => {
                    instructions.push(Instruction::UnaryOperationWithHeld(u_op))
                }
            }
        }

        instructions
    }
}
