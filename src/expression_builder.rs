use crate::{instruction::Instruction, operators::Operator, TypeSystem};

#[derive(Default)]
pub struct ExpressionBuilder<TS: TypeSystem> {
    operator: Option<Operator<TS>>,
    values: [Option<usize>; 2],
}

impl<TS: TypeSystem> ExpressionBuilder<TS> {
    pub fn new() -> ExpressionBuilder<TS> {
        ExpressionBuilder {
            operator: None,
            values: [None; 2],
        }
    }
    pub fn set_value(&mut self, value: usize) -> &mut ExpressionBuilder<TS> {
        match (&self.values[0], &self.values[1]) {
            (None, _) => self.values[0] = Some(value),
            (Some(_), None) => self.values[1] = Some(value),
            _ => (),
        }
        self
    }
    pub fn set_left_value(&mut self, value: usize) -> &mut ExpressionBuilder<TS> {
        self.values[0] = Some(value);
        self
    }

    pub fn set_right_value(&mut self, value: usize) -> &mut ExpressionBuilder<TS> {
        self.values[1] = Some(value);
        self
    }

    pub fn set_operator(&mut self, operator: Operator<TS>) -> &mut ExpressionBuilder<TS> {
        self.operator = Some(operator);
        self
    }

    pub fn build(self) -> Vec<Instruction<TS>> {
        let mut instructions = Vec::new();

        if let Some(offset) = self.values[0] {
            instructions.push(Instruction::MoveToReturn(offset))
        }
        if let Some(offset) = self.values[1] {
            instructions.push(Instruction::MoveRightOperand(offset))
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
