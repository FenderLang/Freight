use std::collections::HashMap;

mod fender_tmp;
pub mod function_builder;

pub struct Function<V: TypeSystem> {
    instructions: Vec<Instruction<V>>,
    args: usize,
}

pub trait TypeSystem {
    type V: Value;
    type B: BinaryOperator<Self::V>;
    type U: UnaryOperator<Self::V>;
    type T;
}

pub struct VMWriter<V: TypeSystem> {
    functions: Vec<Function<V>>,
}

pub enum Instruction<V: TypeSystem> {
    Create(usize, fn(&ExecutionContext<V>) -> V::V),
    Move(usize, usize),
    MoveReturn(usize),
    Invoke(usize, usize, usize),
    InvokeNative(fn(&mut ExecutionContext<V>) -> V::V),
    Return(usize),
    ReturnConstant(V::V),
    UnaryOperation(V::U),
    BinaryOperation(V::B, usize),
}

pub struct ExecutionContext<V: TypeSystem> {
    stack: Vec<V::V>,
    instructions: Vec<Instruction<V>>,
    instruction: usize,
    frames: Vec<usize>,
    frame: usize,
    return_value: V::V,
}

pub trait BinaryOperator<V: Value> {
    fn apply(&self, a: &V, b: &V) -> V;
    fn priority(&self) -> usize;
}

pub trait UnaryOperator<V: Value> {
    fn apply(&self, val: &V) -> V;
}

pub trait Value: Clone + Default {
    type V: TypeSystem;
    fn get_type(&self) -> &<Self::V as TypeSystem>::T;
}

impl<V: TypeSystem> VMWriter<V> {
    fn include_function(&mut self, function: Function<V>) -> usize {
        todo!()
    }

    fn finish(self) -> ExecutionContext<V> {
        todo!()
    }
}

impl<V: TypeSystem> ExecutionContext<V> {
    pub fn new(instructions: Vec<Instruction<V>>, stack_size: usize) -> ExecutionContext<V> {
        ExecutionContext {
            stack: Vec::with_capacity(stack_size),
            instructions,
            instruction: 0,
            frames: vec![],
            frame: 0,
            return_value: Default::default(),
        }
    }

    fn get(&self, offset: usize) -> &V::V {
        &self.stack[self.frame + offset]
    }

    fn get_mut(&mut self, offset: usize) -> &mut V::V {
        &mut self.stack[self.frame + offset]
    }

    fn execute(&mut self, index: usize) {
        let instruction = &self.instructions[index];
        match instruction {
            Instruction::Create(offset, creator) => *self.get_mut(*offset) = creator(self),
            Instruction::Move(from, to) => *self.get_mut(*to) = self.get(*from).clone(),
            Instruction::MoveReturn(to) => {
                *self.get_mut(*to) = std::mem::replace(&mut self.return_value, Default::default())
            }
            Instruction::Invoke(args, stack_size, instruction) => {
                self.frames.push(self.frame);
                self.frame -= args;
                self.instruction = *instruction;
                for _ in 0..stack_size - args {
                    self.stack.push(Default::default());
                }
            }
            Instruction::InvokeNative(func) => self.return_value = func(self),
            Instruction::Return(offset) => {
                self.return_value = self.get(*offset).clone();
                self.frame = self.frames.pop().unwrap();
            }
            Instruction::ReturnConstant(c) => {
                self.return_value = c.clone();
                self.frame = self.frames.pop().unwrap();
            }
            Instruction::UnaryOperation(op) => {
                self.return_value = op.apply(&self.return_value);
            }
            Instruction::BinaryOperation(op, index) => {
                self.return_value = op.apply(&self.return_value, self.get(*index));
            }
        }
    }

    fn run(&mut self) {
        while self.instruction < self.instructions.len() {
            self.execute(self.instruction);
            self.instruction += 1;
        }
    }
}