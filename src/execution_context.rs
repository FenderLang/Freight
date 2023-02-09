use std::fmt::Debug;

use crate::{instruction::Instruction, BinaryOperator, TypeSystem, UnaryOperator};

#[derive(Debug)]
pub struct ExecutionContext<TS: TypeSystem> {
    stack: Vec<TS::Value>,
    instructions: Vec<Instruction<TS>>,
    instruction: usize,
    frames: Vec<usize>,
    frame: usize,
    return_value: TS::Value,
    right_operand: TS::Value,
    popped_value: Option<TS::Value>,
}

impl<TS: TypeSystem> ExecutionContext<TS> {
    pub fn new(instructions: Vec<Instruction<TS>>, stack_size: usize) -> ExecutionContext<TS> {
        ExecutionContext {
            stack: Vec::with_capacity(stack_size),
            instructions,
            instruction: 0,
            frames: vec![],
            frame: 0,
            return_value: Default::default(),
            right_operand: Default::default(),
            popped_value: Default::default(),
        }
    }

    fn get(&self, offset: usize) -> &TS::Value {
        &self.stack[self.frame + offset]
    }

    fn get_mut(&mut self, offset: usize) -> &mut TS::Value {
        &mut self.stack[self.frame + offset]
    }

    fn execute(&mut self, index: usize) {
        use Instruction::*;
        let instruction = &self.instructions[index];
        match instruction {
            Create(offset, creator) => *self.get_mut(*offset) = creator(self),
            Move(from, to) => *self.get_mut(*to) = self.get(*from).clone(),
            MoveFromReturn(to) => *self.get_mut(*to) = std::mem::take(&mut self.return_value),
            MoveToReturn(from) => {
                self.return_value = self.get(*from).clone();
            }
            MoveRightOperand(from) => {
                self.right_operand = self.get(*from).clone();
            }
            Invoke(arg_count, stack_size, instruction) => {
                self.frames.push(self.frame);
                self.frame -= arg_count;
                self.instruction = *instruction;
                for _ in 0..stack_size - arg_count {
                    self.stack.push(Default::default());
                }
            }
            InvokeNative(func) => self.return_value = func(self),
            Return(offset) => {
                self.return_value = self.get(*offset).clone();
                self.frame = self.frames.pop().unwrap();
            }
            ReturnConstant(c) => {
                self.return_value = c.clone();
                self.frame = self.frames.pop().unwrap();
            }
            UnaryOperation(unary_op) => {
                self.return_value = unary_op.apply_1(&self.return_value);
            }
            BinaryOperation(binary_op) => {
                self.return_value = binary_op.apply_2(&self.return_value, &self.right_operand);
            }
            SetReturnRaw(raw_v) => self.return_value = raw_v.clone(),
            SetRightOperandRaw(raw_v) => self.right_operand = raw_v.clone(),
            PushRaw(value) => self.stack.push(value.clone()),
            Pop => self.popped_value = self.stack.pop(),
            Push(from) => self.stack.push(self.get(*from).clone()),
            PushFromReturn => self.stack.push( self.return_value.clone()),
        }
    }

    pub fn run(&mut self) {
        while self.instruction < self.instructions.len() {
            self.execute(self.instruction);
            self.instruction += 1;
        }
    }

    pub fn get_expression_tmp_value_location(&self) -> usize {
        // TODO: @Redempt
        todo!()
    }
}
