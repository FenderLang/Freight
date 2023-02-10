use crate::{instruction::Instruction, TypeSystem};

#[derive(Debug)]
pub struct Function<TS: TypeSystem> {
    instructions: Vec<Instruction<TS>>,
    args: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionRef {
    arg_count: usize,
    stack_size: usize,
    location: usize,
}