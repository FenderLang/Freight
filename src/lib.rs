use std::collections::HashMap;

pub struct Function<V: Clone + Default> {
    instructions: Vec<Instruction<V>>,
    args: usize,
}

pub struct VMWriter<V: Clone + Default> {
    functions: Vec<Function<V>>,
}

impl<V: Clone + Default> VMWriter<V> {
    fn include_function(&mut self, function: Function<V>) -> usize {
        todo!()
    }
    
    fn finish(self) -> ExecutionContext<V> {
        todo!()
    }
}

pub struct ExecutionContext<V: Clone + Default> {
    stack: Vec<V>,
    instructions: Vec<Instruction<V>>,
    instruction: usize,
    frames: Vec<usize>,
    frame: usize,
    return_value: V,
}


impl<V: Clone + Default> ExecutionContext<V> {
    pub fn new(instructions: Vec<Instruction<V>>, stack_size: usize) -> ExecutionContext<V> {
        ExecutionContext {
            stack: Vec::with_capacity(stack_size),
            instructions,
            instruction: 0,
            frames: vec![],
            frame: 0,
            return_value: V::default(),
        }
    }

    fn get(&self, offset: usize) -> &V {
        &self.stack[self.frame + offset]
    }

    fn get_mut(&mut self, offset: usize) -> &mut V {
        &mut self.stack[self.frame + offset]
    }

    fn execute(&mut self, index: usize) {
        let instruction = &self.instructions[index];
        match instruction {
            Instruction::Create(offset, creator) => *self.get_mut(*offset) = creator(self),
            Instruction::Move(from, to) => *self.get_mut(*to) = self.get(*from).clone(),
            Instruction::MoveReturn(to) => *self.get_mut(*to) = std::mem::replace(&mut self.return_value, V::default()),
            Instruction::Invoke(args, stack_size, instruction) => {
                self.frames.push(self.frame);
                self.frame -= args;
                self.instruction = *instruction;
                for _ in 0..stack_size - args {
                    self.stack.push(V::default());
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
        }
    }
    
    fn run(&mut self) {
        while self.instruction < self.instructions.len() {
            self.execute(self.instruction);
            self.instruction += 1;
        }
    }
}

pub enum Instruction<V: Default + Clone> {
    Create(usize, fn(&ExecutionContext<V>) -> V),
    Move(usize, usize),
    MoveReturn(usize),
    Invoke(usize, usize, usize),
    InvokeNative(fn(&mut ExecutionContext<V>) -> V),
    Return(usize),
    ReturnConstant(V),
}