use crate::{instruction::Instruction, TypeSystem};

#[derive(Debug)]
pub struct Function<TS: TypeSystem> {
    instructions: Vec<Instruction<TS>>,
    args: usize,
}
