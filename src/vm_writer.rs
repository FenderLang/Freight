use crate::{execution_context::ExecutionContext, function::Function, TypeSystem};

#[derive(Debug)]
pub struct VMWriter<TS: TypeSystem> {
    functions: Vec<Function<TS>>,
}

impl<TS: TypeSystem> VMWriter<TS> {
    fn include_function(&mut self, function: Function<TS>) -> usize {
        todo!()
    }

    fn finish(self) -> ExecutionContext<TS> {
        todo!()
    }
}