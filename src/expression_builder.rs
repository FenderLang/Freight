use crate::{instruction::Instruction, operators::Operator, TypeSystem};

#[derive(Clone)]

pub enum Operand<TS: TypeSystem> {
    Function(usize, Vec<Operand<TS>>),
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

    pub fn build(self) -> Vec<Instruction<TS>> {
        let mut instructions = Vec::new();

        if let Some(operand) = self.operands.0 {
            match operand {
                Operand::Function(function_addr, args) => todo!(),
                Operand::ValueRef(addr) => instructions.push(Instruction::MoveToReturn(addr)),
                Operand::ValueRaw(val) => instructions.push(Instruction::SetReturnRaw(val)),
            }
        }
        if let Some(operand) = self.operands.1 {
            match operand {
                Operand::Function(function_addr, args) => todo!(),
                Operand::ValueRef(addr) => instructions.push(Instruction::MoveToReturn(addr)),
                Operand::ValueRaw(val) => instructions.push(Instruction::SetReturnRaw(val)),
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
