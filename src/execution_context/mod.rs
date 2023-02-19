use crate::{
    error::FreightError,
    function::{FunctionRef, FunctionType, InvokeNative},
    instruction::{Instruction, InstructionWrapper},
    value::Value,
    BinaryOperator, TypeSystem, UnaryOperator,
};
use std::{fmt::Debug, rc::Rc};

mod location_identifiers;
pub use location_identifiers::*;

#[derive(Debug)]
pub struct ExecutionContext<'a: 'b, 'b, TS: TypeSystem> where Self: 'a {
    pub(crate) stack: Vec<TS::Value>,
    pub(crate) instruction: usize,
    pub(crate) instructions: &'b [Instruction<TS>],
    pub(crate) frames: Vec<usize>,
    pub(crate) call_stack: Vec<usize>,
    pub(crate) frame: usize,
    pub(crate) registers: [TS::Value; 3],
    pub(crate) entry_point: usize,
}

impl<'a, TS: TypeSystem> ExecutionContext<'a, TS> {
    pub fn new(
        instructions: &'a [Instruction<TS>],
        stack_size: usize,
        entry_point: usize,
    ) -> ExecutionContext<'a, TS> {
        ExecutionContext {
            stack: Vec::with_capacity(stack_size),
            instruction: 0,
            instructions,
            frames: vec![],
            call_stack: vec![],
            frame: 0,
            registers: std::array::from_fn(|_| Value::uninitialized_reference()),
            entry_point,
        }
    }

    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    pub fn get_register(&self, register: RegisterId) -> &TS::Value {
        &self.registers[register.id()]
    }

    pub fn get_register_mut(&mut self, register: RegisterId) -> &mut TS::Value {
        &mut self.registers[register.id()]
    }

    pub fn get_stack(&self, offset: usize) -> &TS::Value {
        &self.stack[self.frame + offset]
    }

    pub fn get_stack_mut(&mut self, offset: usize) -> &mut TS::Value {
        &mut self.stack[self.frame + offset]
    }

    pub fn get(&self, location: &Location) -> &TS::Value {
        match location {
            Location::Register(reg) => self.get_register(*reg),
            Location::Stack(offset) => self.get_stack(*offset),
        }
    }

    pub fn get_mut(&mut self, location: &Location) -> &mut TS::Value {
        match location {
            Location::Register(reg) => self.get_register_mut(*reg),
            Location::Stack(offset) => self.get_stack_mut(*offset),
        }
    }

    pub fn set(&mut self, offset: usize, value: TS::Value) {
        self.stack[self.frame + offset] = value;
    }

    pub(crate) fn do_return(&mut self, stack_size: usize) {
        if self.frames.is_empty() {
            self.instruction = usize::max_value();
            return;
        }
        self.frame = self.frames.pop().unwrap();
        self.instruction = self.call_stack.pop().unwrap();
        self.stack.drain((self.stack.len() - stack_size)..);
    }

    pub(crate) fn do_invoke(&mut self, arg_count: usize, stack_size: usize, instruction: usize) {
        self.call_stack.push(self.instruction);
        self.frames.push(self.frame);
        self.instruction = instruction;
        // Subtract 1 to account for the held value slot
        for _ in 0..stack_size - arg_count - 1 {
            self.stack.push(Value::uninitialized_reference());
        }
        self.frame = self.stack.len() - stack_size;
    }

    pub fn print_state(&self) {
        println!("registers: {:?}", self.registers);
        println!("stack: {} frame: {}", self.stack.len(), self.frame);
        println!("frame contents: {:?}", &self.stack[self.frame..]);
        println!("stack context: {}", self.frames.len());
        println!("instruction: {:?}", self.instructions[self.instruction]);
        println!("instruction index: {}", self.instruction);
        println!("-----------");
    }
}

/// execution functionality
impl<'a, TS: TypeSystem> ExecutionContext<'a, TS> {
    pub(crate) fn execute_next(&mut self) -> Result<bool, FreightError> {
        #[cfg(feature = "debug_mode")]
        self.print_state();
        self.instructions[self.instruction].execute(self);
        Ok(self.instruction >= self.instructions.len())
    }

    pub fn call_function(
        &mut self,
        func: FunctionRef<TS>,
        args: Vec<TS::Value>,
    ) -> Result<TS::Value, FreightError> {
        *self.get_register_mut(RegisterId::Return) = func.into();
        let frame_num = self.frames.len();
        self.stack.push(Value::uninitialized_reference());
        let arg_count = args.len();
        self.stack.extend(args);
        Instruction::InvokeDynamic { arg_count }.execute(self)?;
        while self.frames.len() > frame_num {
            self.execute_next()?;
        }
        self.instruction -= 1;
        Ok(std::mem::take(self.get_register_mut(RegisterId::Return)))
    }
}
