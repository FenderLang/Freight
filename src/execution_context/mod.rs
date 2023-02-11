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
    call_stack: Vec<usize>,
    frame: usize,
    registers: [TS::Value; 3],
    entry_point: usize,
    initial_stack_size: usize,
}

impl<TS: TypeSystem> ExecutionContext<TS> {
    pub fn new(instructions: Vec<Instruction<TS>>, stack_size: usize, entry_point: usize) -> ExecutionContext<TS> {
        ExecutionContext {
            stack: Vec::with_capacity(stack_size),
            initial_stack_size: stack_size,
            instructions,
            instruction: 0,
            frames: vec![],
            call_stack: vec![],
            frame: 0,
            registers: Default::default(),
            entry_point,
        }
    }

    pub fn get_register(&self, register: RegisterId) -> &TS::Value {
        &self.registers[register.id()]
    }

    pub fn get(&self, offset: usize) -> &TS::Value {
        &self.stack[self.frame + offset]
    }

    pub fn set(&mut self, offset: usize, value: TS::Value) {
        self.stack[self.frame + offset] = value;
    }

    pub fn get_mut(&mut self, offset: usize) -> &mut TS::Value {
        &mut self.stack[self.frame + offset]
    }

    fn do_return(&mut self, stack_size: usize) {
        self.frame = self.frames.pop().unwrap();
        self.instruction = self.call_stack.pop().unwrap();
        self.stack.drain((self.stack.len() - stack_size)..);
    }

    pub fn execute(&mut self, index: usize) -> bool {
        use Instruction::*;
        let instruction = &self.instructions[index];
        let mut increment_index = true;
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
                // TODO: Argument count checks
                self.call_stack.push(self.instruction);
                self.frames.push(self.frame);
                self.instruction = *instruction;
                // Subtract 1 to account for the held value slot
                for _ in 0..stack_size - arg_count - 1 {
                    self.stack.push(Default::default());
                }
                self.frame = self.stack.len() - stack_size;

                increment_index = false;
            }
            InvokeNative(func) => self.registers[RegisterId::Return.id()] = func(self),
            Return(stack_size) => self.do_return(*stack_size),
            ReturnConstant(c, stack_size) => {
                self.registers[RegisterId::Return.id()] = c.clone();
                self.do_return(*stack_size);
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
        increment_index
    }

    pub fn run(&mut self) {
        self.instruction = self.entry_point;
        self.stack = vec![];
        for _ in 0..self.initial_stack_size {
            self.stack.push(Default::default());
        }
        while self.instruction < self.instructions.len() {
            if self.execute(self.instruction) {
                self.instruction += 1;
            }
        }
    }
}
