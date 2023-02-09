use crate::{
    execution_context::ExecutionContext, instruction::Instruction, operators::Operator, TypeSystem,
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
}

#[derive(Default)]
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
            }
        }
        instructions.push(Instruction::Invoke(function_addr, arg_count, stack_size));
    }

    pub fn build(mut self, execution_context: &mut ExecutionContext<TS>) -> Vec<Instruction<TS>> {
        let mut instructions = Vec::new();

        if let Some(Operand::Function {
            addr,
            args,
            stack_size,
        }) = &mut self.operands.1
        {
            ExpressionBuilder::build_function(
                &mut instructions,
                execution_context,
                *addr,
                args.drain(0..).collect(),
                *stack_size,
            );
            instructions.push(Instruction::MoveFromReturn(
                execution_context.get_expression_tmp_value_location(),
            ))
        }

        if let Some(operand) = self.operands.0 {
            match operand {
                Operand::Function {
                    addr,
                    args,
                    stack_size,
                } => ExpressionBuilder::build_function(
                    &mut instructions,
                    execution_context,
                    addr,
                    args,
                    stack_size,
                ),

                Operand::ValueRef(addr) => instructions.push(Instruction::MoveToReturn(addr)),
                Operand::ValueRaw(val) => instructions.push(Instruction::SetReturnRaw(val)),
            }
        }

        if let Some(operand) = self.operands.1 {
            match operand {
                Operand::Function {
                    addr: _,
                    args: _,
                    stack_size: _,
                } => instructions.push(Instruction::MoveRightOperand(
                    execution_context.get_expression_tmp_value_location(),
                )),
                Operand::ValueRef(addr) => instructions.push(Instruction::MoveRightOperand(addr)),
                Operand::ValueRaw(val) => instructions.push(Instruction::SetRightOperandRaw(val)),
            }
        }

        if let Some(op) = self.operator {
            match op {
                Operator::Binary(b_op) => instructions.push(Instruction::BinaryOperation(b_op)),
                Operator::Unary(u_op) => instructions.push(Instruction::UnaryOperation(u_op)),
            }
        }

        instructions
    }
}
