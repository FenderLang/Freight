use crate::{
    execution_context::register_ids::RegisterId, instruction::Instruction, BinaryOperator,
    TypeSystem, UnaryOperator,
};
use std::fmt::Debug;

pub mod register_ids;

/// Location in stack of the temporary held value used by the expression builder.
pub const HELD_VALUE_LOCATION: usize = 0;

#[derive(Debug)]
pub struct ExecutionContext<TS: TypeSystem> {
    stack: Vec<TS::Value>,
    instructions: Vec<Instruction<TS>>,
    instruction: usize,
    frames: Vec<usize>,
    frame: usize,

    registers: [TS::Value; 3],
}

impl<TS: TypeSystem> ExecutionContext<TS> {
    pub fn new(instructions: Vec<Instruction<TS>>, stack_size: usize) -> ExecutionContext<TS> {
        ExecutionContext {
            stack: Vec::with_capacity(stack_size),
            instructions,
            instruction: 0,
            frames: vec![],
            frame: 0,
            registers: Default::default(),
        }
    }

    fn get(&self, offset: usize) -> &TS::Value {
        &self.stack[self.frame + offset]
    }

    fn set(&mut self, offset: usize, value: TS::Value) {
        self.stack[self.frame + offset] = value;
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
            MoveFromReturn(to) => {
                *self.get_mut(*to) = std::mem::take(&mut self.registers[RegisterId::Return.id()])
            }
            MoveToReturn(from) => {
                self.registers[RegisterId::Return.id()] = self.get(*from).clone();
            }
            MoveRightOperand(from) => {
                self.registers[RegisterId::RightOperand.id()] = self.get(*from).clone();
            }
            Invoke(arg_count, stack_size, instruction) => {
                self.frames.push(self.frame);
                self.frame -= arg_count;
                self.instruction = *instruction;
                for _ in 0..stack_size - arg_count {
                    self.stack.push(Default::default());
                }
            }
            InvokeNative(func) => self.registers[RegisterId::Return.id()] = func(self),
            Return => self.frame = self.frames.pop().unwrap(),
            ReturnConstant(c) => {
                self.registers[RegisterId::Return.id()] = c.clone();
                self.frame = self.frames.pop().unwrap();
            }
            UnaryOperation(unary_op) => {
                self.registers[RegisterId::Return.id()] =
                    unary_op.apply_1(&self.registers[RegisterId::Return.id()]);
            }
            BinaryOperation(binary_op) => {
                self.registers[RegisterId::Return.id()] = binary_op.apply_2(
                    &self.registers[RegisterId::Return.id()],
                    &self.registers[RegisterId::RightOperand.id()],
                );
            }
            SetReturnRaw(raw_v) => self.registers[RegisterId::Return.id()] = raw_v.clone(),
            SetRightOperandRaw(raw_v) => {
                self.registers[RegisterId::RightOperand.id()] = raw_v.clone()
            }
            PushRaw(value) => self.stack.push(value.clone()),
            #[cfg(feature="popped_register")]
            Pop => self.registers[RegisterId::Popped.id()] = self.stack.pop().unwrap_or_default(),
            #[cfg(not(feature="popped_register"))]
            Pop => self.registers[RegisterId::Return.id()] = self.stack.pop().unwrap_or_default(),
            Push(from) => self.stack.push(self.get(*from).clone()),
            PushFromReturn => self
                .stack
                .push(self.registers[RegisterId::Return.id()].clone()),
            UnaryOperationWithHeld(unary_op) => {
                self.registers[RegisterId::Return.id()] =
                    unary_op.apply_1(&self.get(HELD_VALUE_LOCATION))
            }
            BinaryOperationWithHeld(binary_op) => {
                self.registers[RegisterId::Return.id()] = binary_op.apply_2(
                    self.get(HELD_VALUE_LOCATION),
                    &self.registers[RegisterId::RightOperand.id()],
                )
            }
            SetHeldRaw(raw_v) => self.set(HELD_VALUE_LOCATION, raw_v.clone()),
        }
    }

    pub fn run(&mut self) {
        while self.instruction < self.instructions.len() {
            self.execute(self.instruction);
            self.instruction += 1;
        }
    }
}
